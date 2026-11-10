import {
  $,
  model,
  t,
  node,
  button,
  iconButton,
  request,
  report,
  send,
  showView,
  toast,
} from "/control-api.js";
import { icon } from "/control-icons.js";

export interface LibraryItem {
  id: string | number;
  kind: string;
  title: string;
  subtitle?: string;
  folder?: string;
}

export interface LibraryResult {
  folders: string[];
  items: LibraryItem[];
}

export interface SongResource {
  id?: number | string | null;
  title?: string;
  artist?: string;
  content?: string;
}

let kind = "song";
let searchTimer: ReturnType<typeof setTimeout> | undefined;
let requestId = 0;
let selected: LibraryItem | undefined;
const openFolders = new Set<string>();

export async function refreshLibrary(): Promise<void> {
  if (kind === "bible") return;
  const id = ++requestId;
  const countEl = $("library-count");
  if (countEl) countEl.textContent = t("loading");
  try {
    const searchInput = $<HTMLInputElement>("library-search");
    const folderSelect = $<HTMLSelectElement>("library-folder");
    const params = new URLSearchParams({
      kind,
      q: searchInput ? searchInput.value : "",
    });
    if (folderSelect && folderSelect.value !== "*") {
      params.set("folder", folderSelect.value);
    }
    const result = await request<LibraryResult>("library?" + params);
    if (id !== requestId) return;
    const folder = folderSelect ? folderSelect.value : "*";
    const options: HTMLOptionElement[] = [
      node("option", t("all_folders")),
      node("option", t("root_folder")),
    ];
    options[0].value = "*";
    options[1].value = "";
    result.folders.forEach((path) => {
      const option = node("option", path);
      option.value = path;
      options.push(option);
    });
    if (folderSelect) {
      folderSelect.replaceChildren(...options);
      folderSelect.value = options.some((option) => option.value === folder)
        ? folder
        : "*";
    }
    if (countEl) {
      countEl.textContent = String(result.items.length);
      countEl.title = result.items.length + " " + t("items");
    }
    const itemsEl = $("library-items");
    if (!itemsEl) return;
    itemsEl.replaceChildren();
    let visible = 0;
    const groups = new Map<string, HTMLElement>();
    const ordered = [...result.items].sort((a, b) =>
      a.folder && b.folder
        ? a.folder.localeCompare(b.folder)
        : a.folder
          ? -1
          : b.folder
            ? 1
            : 0,
    );
    const more = button(t("load_more"), () => append());
    function append(): void {
      more.remove();
      ordered.slice(visible, visible + 100).forEach((item) => {
        if (!item.folder) {
          itemsEl?.append(resourceRow(item));
          return;
        }
        if (!groups.has(item.folder)) {
          const group = node("div", undefined, "native-folder");
          const children = node("div", undefined, "folder-children");
          const key = kind + ":" + item.folder;
          const header = button(
            "",
            () => {
              if (openFolders.has(key)) openFolders.delete(key);
              else openFolders.add(key);
              update();
            },
            "folder-row",
          );
          const update = () => {
            const opened =
              openFolders.has(key) || !!searchInput?.value;
            header.replaceChildren(
              icon(opened ? "down" : "next"),
              icon("folder"),
              node("span", item.folder),
            );
            header.setAttribute("aria-expanded", String(opened));
            children.hidden = !opened;
          };
          update();
          group.append(header, children);
          groups.set(item.folder, children);
          itemsEl?.append(group);
        }
        groups.get(item.folder)!.append(resourceRow(item));
      });
      visible += 100;
      if (visible < result.items.length) itemsEl?.append(more);
    }
    append();
    if (!result.items.length) {
      itemsEl.append(node("p", t("empty_library"), "empty"));
    }
    highlightLibrary();
  } catch (error) {
    if (id === requestId) {
      report(error);
      if (countEl) countEl.textContent = "";
    }
  }
}

function resourceRow(item: LibraryItem): HTMLElement {
  const row = node("article", undefined, "resource-row");
  row.dataset.id = String(item.id);
  const activate = async (project: boolean, control?: HTMLButtonElement) => {
    selected = item;
    const result = await send(
      { action: "select", kind: item.kind, id: item.id, project },
      project ? control : undefined,
    );
    if (result) {
      highlightLibrary();
      showView("operation", project ? "live" : "preview");
    }
  };
  const title = button(
    "",
    (control) => activate(false, control),
    "resource-title",
  );
  const label = node("span");
  label.append(node("strong", item.title), node("small", item.subtitle));
  const mark = node("span", undefined, "resource-mark");
  mark.append(icon(item.kind === "song" ? "music" : item.kind));
  title.append(mark, label);
  title.title = item.title;
  title.ondblclick = () => activate(true);
  const actions = node("div", undefined, "row-actions");
  actions.append(
    iconButton(t("add_playlist"), "plus", async (control) => {
      if (
        await send(
          { action: "playlist_add", kind: item.kind, id: item.id },
          control,
        )
      ) {
        toast(t("added"));
      }
    }),
    iconButton(t("pin"), "pin", async (control) => {
      if (
        await send(
          { action: "pin", kind: item.kind, id: item.id },
          control,
        )
      ) {
        toast(t("pinned"));
      }
    }),
    iconButton(
      t("project"),
      "play",
      (control) => activate(true, control),
      "primary",
    ),
  );
  row.append(title, actions);
  return row;
}

export function highlightLibrary(): void {
  document.querySelectorAll<HTMLElement>(".resource-row").forEach((row) => {
    const state = model.state;
    row.classList.toggle(
      "selected",
      kind === "song"
        ? row.dataset.id === String(state?.preview?.song_id)
        : kind === selected?.kind &&
            row.dataset.id === String(selected?.id) &&
            state?.preview?.title === selected?.title,
    );
  });
}

export function setupLibrary(onBible: (refresh?: boolean) => void): void {
  document.querySelectorAll<HTMLElement>("[data-kind]").forEach(
    (tab) =>
      (tab.onclick = () => {
        kind = tab.dataset.kind || "song";
        ++requestId;
        document
          .querySelectorAll<HTMLElement>("[data-kind]")
          .forEach((item) => item.classList.toggle("active", item === tab));
        const bibleEl = $("bible");
        if (bibleEl) bibleEl.hidden = kind !== "bible";
        const catalogEl = $("catalog");
        if (catalogEl) catalogEl.hidden = kind === "bible";
        if (kind === "bible") {
          onBible();
        } else {
          const searchInput = $<HTMLInputElement>("library-search");
          if (searchInput) searchInput.value = "";
          const folderSelect = $<HTMLSelectElement>("library-folder");
          if (folderSelect) folderSelect.value = "*";
          refreshLibrary();
        }
      }),
  );
  const searchInput = $<HTMLInputElement>("library-search");
  if (searchInput) {
    searchInput.oninput = () => {
      ++requestId;
      clearTimeout(searchTimer);
      searchTimer = setTimeout(refreshLibrary, 90);
    };
  }
  const folderSelect = $<HTMLSelectElement>("library-folder");
  if (folderSelect) {
    folderSelect.onchange = () => refreshLibrary();
  }
  const refreshBtn = $("library-refresh");
  if (refreshBtn) {
    refreshBtn.onclick = () =>
      kind === "bible" ? onBible(true) : refreshLibrary();
  }
  const newSongBtn = $("song-new");
  if (newSongBtn) newSongBtn.onclick = () => openSongEditor();
  const editSongBtn = $("edit-song");
  if (editSongBtn) {
    editSongBtn.onclick = async () => {
      const id = model.state?.preview?.song_id;
      if (!id) return;
      try {
        openSongEditor(
          await request<SongResource>(
            "resource?" + new URLSearchParams({ kind: "song", id: String(id) }),
          ),
        );
      } catch (error) {
        report(error);
      }
    };
  }
  const songForm = $<HTMLFormElement>("song-form");
  if (songForm) {
    songForm.onsubmit = async (event: SubmitEvent) => {
      event.preventDefault();
      const idVal = $<HTMLInputElement>("song-id")?.value;
      const result = await send(
        {
          action: "save_song",
          id: idVal ? Number(idVal) : null,
          title: $<HTMLInputElement>("song-title")?.value || "",
          artist: $<HTMLInputElement>("song-artist")?.value || "",
          content: $<HTMLTextAreaElement>("song-content")?.value || "",
        },
        event.submitter as HTMLButtonElement | null,
      );
      if (result) {
        $<HTMLDialogElement>("song-dialog")?.close();
        refreshLibrary();
        toast(t("saved"));
      }
    };
  }
}

function openSongEditor(song: SongResource = {}): void {
  const idInput = $<HTMLInputElement>("song-id");
  if (idInput) idInput.value = song.id ? String(song.id) : "";
  const titleInput = $<HTMLInputElement>("song-title");
  if (titleInput) titleInput.value = song.title || "";
  const artistInput = $<HTMLInputElement>("song-artist");
  if (artistInput) artistInput.value = song.artist || "";
  const contentInput = $<HTMLTextAreaElement>("song-content");
  if (contentInput) contentInput.value = song.content || "";
  const dialog = $<HTMLDialogElement>("song-dialog");
  dialog?.querySelector(".dialog-error")?.remove();
  dialog?.showModal();
  titleInput?.focus();
}

