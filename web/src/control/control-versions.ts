import { $, model, node, button, iconButton, send, t } from "/control-api.js";
import { changeBibleVersion } from "/control-bible.js";
import type { ControlState } from "../types/state.js";

export function createVersionBar(): HTMLElement {
  const bar = node("div", undefined, "native-version-bar");
  bar.id = "native-version-bar";
  const pills = node("div", undefined, "native-version-pills");
  const dialog = node("dialog", undefined, "native-version-picker");
  dialog.id = "native-version-picker";
  const header = node("div", undefined, "panel-heading");
  const title = node("h2"); title.dataset.i18n = "bible_version";
  header.append(title, iconButton(t("close"), "close", () => dialog.close()));
  const search = node("input"); search.type = "search"; search.dataset.placeholder = "search";
  search.id = "version-search"; search.setAttribute("aria-label", t("bible_version"));
  const languages = node("select"); languages.setAttribute("aria-label", t("language"));
  const list = node("div", undefined, "native-version-list");
  let state: ControlState | null = null;
  let signature = "";
  const renderList = () => {
    const query = search.value.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
    const versions = state?.bible_versions || [];
    list.replaceChildren(...versions.filter(v => (!languages.value || v.language === languages.value)
      && `${v.id} ${v.name}`.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase().includes(query)).map(v => {
      const row = node("div", undefined, "native-version-row");
      const choose = button("", async () => { await changeBibleVersion(v.id); dialog.close(); });
      choose.classList.toggle("active", v.id === state?.bible_version);
      choose.append(node("strong", v.id), node("small", v.name));
      const pinned = state?.bible_shortcuts?.includes(v.id);
      const pin = iconButton(t(pinned ? "remove" : "pin"), pinned ? "check_circle" : "plus", () =>
        send({action: "bible_shortcut", id: v.id, operation: pinned ? "remove" : "add"}));
      pin.setAttribute("aria-pressed", String(!!pinned));
      row.append(choose, pin); return row;
    }));
    if (!list.childElementCount) list.append(node("p", t("empty_library"), "empty"));
  };
  const open = () => { search.value = ""; renderList(); dialog.showModal(); search.focus(); };
  const add = iconButton(t("bible_version"), "plus", open); add.id = "version-picker-open";
  bar.append(pills, add);
  dialog.append(header, search, languages, list);
  document.body.append(dialog);
  search.oninput = renderList; languages.onchange = renderList;
  document.addEventListener("control-versions", event => {
    state = (event as CustomEvent<ControlState>).detail;
    const key = JSON.stringify([state.bible_versions, state.bible_shortcuts, state.bible_version, state.language]);
    if (signature === key) return;
    signature = key;
    title.textContent = t("bible_version"); search.placeholder = t("search"); add.title = t("bible_version");
    const selectedLanguage = languages.value;
    const all = node("option", t("all_languages")); all.value = "";
    languages.replaceChildren(all, ...Array.from(new Set(state.bible_versions?.map(v => v.language))).sort().map(language => {
      const option = node("option", language.toUpperCase()); option.value = language; return option;
    }));
    languages.value = selectedLanguage;
    pills.replaceChildren(...(state.bible_shortcuts || []).map(id => {
      const version = state?.bible_versions?.find(v => v.id === id);
      const pill = button(id.replace(/^[a-z]{2}_/i, ""), () => changeBibleVersion(id));
      pill.title = version?.name || id; pill.dataset.version = id; pill.draggable = true;
      pill.classList.toggle("active", id === state?.bible_version);
      pill.setAttribute("aria-pressed", String(id === state?.bible_version));
      pill.ondragstart = e => e.dataTransfer?.setData("application/x-bible-version", id);
      pill.ondragover = e => { if (e.dataTransfer?.types.includes("application/x-bible-version")) e.preventDefault(); };
      pill.ondrop = e => { e.preventDefault(); const source = e.dataTransfer?.getData("application/x-bible-version"); if (source && source !== id) send({action: "bible_shortcut", id: source, target: id, operation: "move"}); };
      return pill;
    }));
    // An unpinned active version remains selectable without silently repinning it.
    if (!state.bible_shortcuts?.includes(state.bible_version || "")) {
      const current = button(state.bible_version || t("bible_version"), open, "active"); pills.prepend(current);
    }
    if (dialog.open) renderList();
  });
  return bar;
}
