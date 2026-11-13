import { $, model, t, node, button, iconButton, request, report, send, toast } from "/control-api.js";

interface SongResource { id?: number | string | null; title?: string; artist?: string; content?: string; folder?: string; tags?: string; }

export function setupSongEditor(refresh: () => Promise<void>): void {
  const dialog = $<HTMLDialogElement>("song-dialog");
  const form = $<HTMLFormElement>("song-form");
  const text = $<HTMLTextAreaElement>("song-content");
  const strophes = $("song-strophes");
  let original = "";
  const values = () => ["song-title", "song-artist", "song-folder", "song-tags", "song-content"].map(id => $<HTMLInputElement>(id).value);
  const dirty = () => JSON.stringify(values()) !== original;
  const blocks = () => text.value.replace(/\r\n?/g, "\n").split(/\n\s*\n/).filter(s => s.trim());
  const update = () => {
    const parts = blocks();
    $("song-strophe-count").textContent = `${parts.length} ${t("strophe")}`;
    strophes.replaceChildren(...parts.map((part, index) => {
      const card = node("article", undefined, "song-strophe"); card.draggable = true;
      const select = button("", () => {
        let offset = 0;
        for (let i = 0; i <= index; i++) {
          offset = text.value.indexOf(parts[i], offset);
          if (i < index) offset += parts[i].length;
        }
        text.focus(); text.setSelectionRange(offset, offset + part.length);
      });
      const label = part.split("\n").find(s => s.trim().startsWith("#*"))?.trim().slice(2) || `${t("strophe")} ${index + 1}`;
      select.append(node("strong", label), node("span", part.split("\n").filter(s => !s.trim().startsWith("//") && !s.trim().startsWith("#*")).join("\n")));
      const tools = node("div", undefined, "song-strophe-tools");
      const commit = (next: string[]) => { text.value = next.join("\n\n"); update(); };
      const up = iconButton(t("move_up"), "up", () => { const next = [...parts]; [next[index - 1], next[index]] = [next[index], next[index - 1]]; commit(next); }); up.disabled = index === 0;
      const down = iconButton(t("move_down"), "down", () => { const next = [...parts]; [next[index + 1], next[index]] = [next[index], next[index + 1]]; commit(next); }); down.disabled = index === parts.length - 1;
      tools.append(up, down, iconButton(t("duplicate"), "plus", () => commit([...parts.slice(0, index + 1), part, ...parts.slice(index + 1)])),
        iconButton(t("remove"), "trash", () => commit(parts.filter((_, i) => i !== index))));
      card.ondragstart = e => e.dataTransfer?.setData("application/x-song-strophe", String(index));
      card.ondragover = e => { if (e.dataTransfer?.types.includes("application/x-song-strophe")) e.preventDefault(); };
      card.ondrop = e => { e.preventDefault(); const raw = e.dataTransfer?.getData("application/x-song-strophe"); if (!raw) return; const from = Number(raw); if (Number.isInteger(from) && from >= 0 && from < parts.length && from !== index) { const next = [...parts]; next.splice(index, 0, next.splice(from, 1)[0]); commit(next); } };
      card.append(select, tools); return card;
    }));
  };
  const insert = (before: string, after = "") => {
    const start = text.selectionStart, end = text.selectionEnd;
    const selected = text.value.slice(start, end);
    text.setRangeText(before + selected + after, start, end, "select");
    text.focus(); update();
  };
  const tools = $("song-format-tools");
  for (const [label, before, after] of [["bold", "**", "**"], ["italic", "*", "*"], ["underline", "__", "__"], ["emphasis", "==", "=="], ["conditional_break", " | ", ""], ["new_strophe", "\n\n#*verso\n", ""]]) {
    const control = button(t(label), () => insert(before, after)); control.dataset.i18n = label; tools.append(control);
  }
  const color = node("input"); color.type = "color"; color.value = "#ffffff"; color.title = t("text_color");
  color.onchange = () => insert(`[c: ${color.value}]`, "[/c]"); tools.append(color);
  text.addEventListener("input", update);
  const open = (song: SongResource = {}) => {
    $<HTMLInputElement>("song-id").value = song.id ? String(song.id) : "";
    for (const [id, value] of [["title", song.title], ["artist", song.artist], ["folder", song.folder], ["tags", song.tags], ["content", song.content]]) $<HTMLInputElement>(`song-${id}`).value = value || "";
    dialog.querySelector(".dialog-error")?.remove();
    original = JSON.stringify(values()); update(); dialog.showModal(); $<HTMLInputElement>("song-title").focus();
  };
  $("song-new").onclick = () => open();
  $("edit-song").onclick = async () => {
    const id = model.state?.preview?.song_id; if (!id) return;
    try { open(await request<SongResource>("resource?" + new URLSearchParams({kind: "song", id: String(id)}))); } catch (error) { report(error); }
  };
  dialog.addEventListener("cancel", event => { if (dirty() && !confirm(t("discard_song"))) event.preventDefault(); });
  dialog.addEventListener("click", event => { if ((event.target as HTMLElement).closest("[data-close]") && dirty() && !confirm(t("discard_song"))) { event.preventDefault(); event.stopImmediatePropagation(); } }, true);
  form.addEventListener("keydown", event => { if ((event.ctrlKey || event.metaKey) && event.key === "Enter") { event.preventDefault(); form.requestSubmit(); } });
  form.onsubmit = async event => {
    event.preventDefault();
    if (!form.reportValidity()) return;
    const id = $<HTMLInputElement>("song-id").value;
    const result = await send({action: "save_song", id: id ? Number(id) : null,
      title: $<HTMLInputElement>("song-title").value, artist: $<HTMLInputElement>("song-artist").value,
      folder: $<HTMLInputElement>("song-folder").value, tags: $<HTMLInputElement>("song-tags").value, content: text.value}, event.submitter as HTMLButtonElement | null);
    if (result) { original = JSON.stringify(values()); dialog.close(); await refresh(); toast(t("saved")); }
  };
}
