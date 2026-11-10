import { $, model, t, node, button, iconButton, request, report, send, showView, toast, } from "/control-api.js";
import { icon } from "/control-icons.js";
let kind = "song";
let searchTimer;
let requestId = 0;
let selected;
const openFolders = new Set();
export async function refreshLibrary() {
    if (kind === "bible")
        return;
    const id = ++requestId;
    const countEl = $("library-count");
    if (countEl)
        countEl.textContent = t("loading");
    try {
        const searchInput = $("library-search");
        const folderSelect = $("library-folder");
        const params = new URLSearchParams({
            kind,
            q: searchInput ? searchInput.value : "",
        });
        if (folderSelect && folderSelect.value !== "*") {
            params.set("folder", folderSelect.value);
        }
        const result = await request("library?" + params);
        if (id !== requestId)
            return;
        const folder = folderSelect ? folderSelect.value : "*";
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
        if (!itemsEl)
            return;
        itemsEl.replaceChildren();
        let visible = 0;
        const groups = new Map();
        const ordered = [...result.items].sort((a, b) => a.folder && b.folder
            ? a.folder.localeCompare(b.folder)
            : a.folder
                ? -1
                : b.folder
                    ? 1
                    : 0);
        const more = button(t("load_more"), () => append());
        function append() {
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
                    const header = button("", () => {
                        if (openFolders.has(key))
                            openFolders.delete(key);
                        else
                            openFolders.add(key);
                        update();
                    }, "folder-row");
                    const update = () => {
                        const opened = openFolders.has(key) || !!searchInput?.value;
                        header.replaceChildren(icon(opened ? "down" : "next"), icon("folder"), node("span", item.folder));
                        header.setAttribute("aria-expanded", String(opened));
                        children.hidden = !opened;
                    };
                    update();
                    group.append(header, children);
                    groups.set(item.folder, children);
                    itemsEl?.append(group);
                }
                groups.get(item.folder).append(resourceRow(item));
            });
            visible += 100;
            if (visible < result.items.length)
                itemsEl?.append(more);
        }
        append();
        if (!result.items.length) {
            itemsEl.append(node("p", t("empty_library"), "empty"));
        }
        highlightLibrary();
    }
    catch (error) {
        if (id === requestId) {
            report(error);
            if (countEl)
                countEl.textContent = "";
        }
    }
}
function resourceRow(item) {
    const row = node("article", undefined, "resource-row");
    row.dataset.id = String(item.id);
    const activate = async (project, control) => {
        selected = item;
        const result = await send({ action: "select", kind: item.kind, id: item.id, project }, project ? control : undefined);
        if (result) {
            highlightLibrary();
            showView("operation", project ? "live" : "preview");
        }
    };
    const title = button("", (control) => activate(false, control), "resource-title");
    const label = node("span");
    label.append(node("strong", item.title), node("small", item.subtitle));
    const mark = node("span", undefined, "resource-mark");
    mark.append(icon(item.kind === "song" ? "music" : item.kind));
    title.append(mark, label);
    title.title = item.title;
    title.ondblclick = () => activate(true);
    const actions = node("div", undefined, "row-actions");
    actions.append(iconButton(t("add_playlist"), "plus", async (control) => {
        if (await send({ action: "playlist_add", kind: item.kind, id: item.id }, control)) {
            toast(t("added"));
        }
    }), iconButton(t("pin"), "pin", async (control) => {
        if (await send({ action: "pin", kind: item.kind, id: item.id }, control)) {
            toast(t("pinned"));
        }
    }), iconButton(t("project"), "play", (control) => activate(true, control), "primary"));
    row.append(title, actions);
    return row;
}
export function highlightLibrary() {
    document.querySelectorAll(".resource-row").forEach((row) => {
        const state = model.state;
        row.classList.toggle("selected", kind === "song"
            ? row.dataset.id === String(state?.preview?.song_id)
            : kind === selected?.kind &&
                row.dataset.id === String(selected?.id) &&
                state?.preview?.title === selected?.title);
    });
}
export function setupLibrary(onBible) {
    document.querySelectorAll("[data-kind]").forEach((tab) => (tab.onclick = () => {
        kind = tab.dataset.kind || "song";
        ++requestId;
        document
            .querySelectorAll("[data-kind]")
            .forEach((item) => item.classList.toggle("active", item === tab));
        const bibleEl = $("bible");
        if (bibleEl)
            bibleEl.hidden = kind !== "bible";
        const catalogEl = $("catalog");
        if (catalogEl)
            catalogEl.hidden = kind === "bible";
        if (kind === "bible") {
            onBible();
        }
        else {
            const searchInput = $("library-search");
            if (searchInput)
                searchInput.value = "";
            const folderSelect = $("library-folder");
            if (folderSelect)
                folderSelect.value = "*";
            refreshLibrary();
        }
    }));
    const searchInput = $("library-search");
    if (searchInput) {
        searchInput.oninput = () => {
            ++requestId;
            clearTimeout(searchTimer);
            searchTimer = setTimeout(refreshLibrary, 90);
        };
    }
    const folderSelect = $("library-folder");
    if (folderSelect) {
        folderSelect.onchange = () => refreshLibrary();
    }
    const refreshBtn = $("library-refresh");
    if (refreshBtn) {
        refreshBtn.onclick = () => kind === "bible" ? onBible(true) : refreshLibrary();
    }
    const newSongBtn = $("song-new");
    if (newSongBtn)
        newSongBtn.onclick = () => openSongEditor();
    const editSongBtn = $("edit-song");
    if (editSongBtn) {
        editSongBtn.onclick = async () => {
            const id = model.state?.preview?.song_id;
            if (!id)
                return;
            try {
                openSongEditor(await request("resource?" + new URLSearchParams({ kind: "song", id: String(id) })));
            }
            catch (error) {
                report(error);
            }
        };
    }
    const songForm = $("song-form");
    if (songForm) {
        songForm.onsubmit = async (event) => {
            event.preventDefault();
            const idVal = $("song-id")?.value;
            const result = await send({
                action: "save_song",
                id: idVal ? Number(idVal) : null,
                title: $("song-title")?.value || "",
                artist: $("song-artist")?.value || "",
                content: $("song-content")?.value || "",
            }, event.submitter);
            if (result) {
                $("song-dialog")?.close();
                refreshLibrary();
                toast(t("saved"));
            }
        };
    }
}
function openSongEditor(song = {}) {
    const idInput = $("song-id");
    if (idInput)
        idInput.value = song.id ? String(song.id) : "";
    const titleInput = $("song-title");
    if (titleInput)
        titleInput.value = song.title || "";
    const artistInput = $("song-artist");
    if (artistInput)
        artistInput.value = song.artist || "";
    const contentInput = $("song-content");
    if (contentInput)
        contentInput.value = song.content || "";
    const dialog = $("song-dialog");
    dialog?.querySelector(".dialog-error")?.remove();
    dialog?.showModal();
    titleInput?.focus();
}
