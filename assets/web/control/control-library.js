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

let kind = "song",
  searchTimer,
  requestId = 0,
  selected;
const openFolders = new Set();
export async function refreshLibrary() {
  if (kind === "bible") return;
  const id = ++requestId;
  $("library-count").textContent = t("loading");
  try {
    const params = new URLSearchParams({ kind, q: $("library-search").value });
    if ($("library-folder").value !== "*")
      params.set("folder", $("library-folder").value);
    const result = await request("library?" + params);
    if (id !== requestId) return;
    const folder = $("library-folder").value;
    const options = [
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
    $("library-folder").replaceChildren(...options);
    $("library-folder").value = options.some(
      (option) => option.value === folder,
    )
      ? folder
      : "*";
    $("library-count").textContent = result.items.length;
    $("library-count").title = result.items.length + " " + t("items");
    $("library-items").replaceChildren();
    let visible = 0;
    const groups = new Map();
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
    function append() {
      more.remove();
      ordered.slice(visible, visible + 100).forEach((item) => {
        if (!item.folder) {
          $("library-items").append(resourceRow(item));
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
            const opened = openFolders.has(key) || !!$("library-search").value;
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
          $("library-items").append(group);
        }
        groups.get(item.folder).append(resourceRow(item));
      });
      visible += 100;
      if (visible < result.items.length) $("library-items").append(more);
    }
    append();
    if (!result.items.length)
      $("library-items").append(node("p", t("empty_library"), "empty"));
    highlightLibrary();
  } catch (error) {
    if (id === requestId) {
      report(error);
      $("library-count").textContent = "";
    }
  }
}

function resourceRow(item) {
  const row = node("article", undefined, "resource-row");
  row.dataset.id = item.id;
  const activate = async (project, control) => {
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
      )
        toast(t("added"));
    }),
    iconButton(t("pin"), "pin", async (control) => {
      if (await send({ action: "pin", kind: item.kind, id: item.id }, control))
        toast(t("pinned"));
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

export function highlightLibrary() {
  document.querySelectorAll(".resource-row").forEach((row) => {
    const state = model.state;
    row.classList.toggle(
      "selected",
      kind === "song"
        ? row.dataset.id === String(state?.preview.song_id)
        : kind === selected?.kind &&
            row.dataset.id === selected?.id &&
            state?.preview.title === selected?.title,
    );
  });
}

export function setupLibrary(onBible) {
  document.querySelectorAll("[data-kind]").forEach(
    (tab) =>
      (tab.onclick = () => {
        kind = tab.dataset.kind;
        ++requestId;
        document
          .querySelectorAll("[data-kind]")
          .forEach((item) => item.classList.toggle("active", item === tab));
        $("bible").hidden = kind !== "bible";
        $("catalog").hidden = kind === "bible";
        if (kind === "bible") onBible();
        else {
          $("library-search").value = "";
          $("library-folder").value = "*";
          refreshLibrary();
        }
      }),
  );
  $("library-search").oninput = () => {
    ++requestId;
    clearTimeout(searchTimer);
    searchTimer = setTimeout(refreshLibrary, 90);
  };
  $("library-folder").onchange = refreshLibrary;
  $("library-refresh").onclick = () =>
    kind === "bible" ? onBible(true) : refreshLibrary();
  $("song-new").onclick = () => openSongEditor();
  $("edit-song").onclick = async () => {
    const id = model.state?.preview.song_id;
    if (!id) return;
    try {
      openSongEditor(
        await request("resource?" + new URLSearchParams({ kind: "song", id })),
      );
    } catch (error) {
      report(error);
    }
  };
  $("song-form").onsubmit = async (event) => {
    event.preventDefault();
    const result = await send(
      {
        action: "save_song",
        id: $("song-id").value ? Number($("song-id").value) : null,
        title: $("song-title").value,
        artist: $("song-artist").value,
        content: $("song-content").value,
      },
      event.submitter,
    );
    if (result) {
      $("song-dialog").close();
      refreshLibrary();
      toast(t("saved"));
    }
  };
}

function openSongEditor(song = {}) {
  $("song-id").value = song.id || "";
  $("song-title").value = song.title || "";
  $("song-artist").value = song.artist || "";
  $("song-content").value = song.content || "";
  $("song-dialog").querySelector(".dialog-error")?.remove();
  $("song-dialog").showModal();
  $("song-title").focus();
}
