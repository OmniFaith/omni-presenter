import {
  $,
  model,
  t,
  node,
  button,
  iconButton,
  request,
  send,
  showView,
  revealInList,
  report,
} from "/control-api.js";
import { icon } from "/control-icons.js";
import { openBible, selectBibleReference } from "/control-bible.js";

let books = [],
  version = "",
  expanded,
  chapter,
  chapterKey = "",
  start = 1,
  end = 1,
  anchor = 1,
  panel,
  tree;

function reference(verse = 1, last = verse) {
  return `${chapter.book} ${chapter.number}:${verse}${verse === last ? "" : "-" + last}`;
}
async function prepare(project = false) {
  if (!chapter) return;
  await send({
    action: "bible",
    reference: reference(start, end),
    version: $("bible-version").value,
    merge: $("bible-merge").checked,
    project,
    playlist: false,
  });
}
function loadChapter(book, number) {
  return selectBibleReference(`${book} ${number}:1`).then(() => {
    if (matchMedia("(max-width:860px)").matches)
      showView("operation", "preview");
  });
}

function renderBooks(follow = false) {
  tree.replaceChildren(
    ...books.map((book) => {
      const group = node("div", undefined, "native-book");
      const choose = button("", () => {
        expanded = expanded === book.name ? null : book.name;
        renderBooks();
      });
      choose.append(
        icon(expanded === book.name ? "down" : "next"),
        node("span", book.name),
      );
      choose.classList.toggle("active", chapter?.book === book.name);
      group.append(choose);
      if (expanded === book.name) {
        const chapters = node("div", undefined, "native-chapter-grid");
        for (let number = 1; number <= book.chapters; number++) {
          const item = button(String(number), () =>
            loadChapter(book.name, number),
          );
          item.classList.toggle(
            "active",
            chapter?.book === book.name && chapter.number === number,
          );
          chapters.append(item);
        }
        group.append(chapters);
      }
      return group;
    }),
  );
  if (follow)
    revealInList(tree, tree.querySelector(".native-book > button.active"));
}

function renderVerses(result) {
  const previousBook = chapter?.book;
  chapter = result.chapter;
  if (!chapter) return;
  if (previousBook !== chapter.book) expanded = chapter.book;
  const range = result.selection?.reference.match(/:(\d+)(?:[–-](\d+))?$/);
  start = range ? Number(range[1]) : 1;
  end = range ? Number(range[2] || range[1]) : start;
  $("native-chapter-title").textContent = chapter.book + " " + chapter.number;
  $("native-chapter-previous").disabled = chapter.number <= 1;
  $("native-chapter-next").disabled = chapter.number >= chapter.total;
  const key = JSON.stringify(chapter);
  if (key !== chapterKey) {
    chapterKey = key;
    $("native-verses").replaceChildren(
      ...chapter.verses.map((text, index) => {
        const verse = index + 1;
        const row = button("", () => {}, "native-verse");
        row.dataset.verse = verse;
        row.replaceChildren(
          node("span", String(verse), "native-verse-number"),
          node("span", text),
        );
        row.onclick = (event) => {
          if (!event.shiftKey) anchor = verse;
          selectBibleReference(
            reference(Math.min(anchor, verse), Math.max(anchor, verse)),
          );
        };
        row.ondblclick = async () => {
          await selectBibleReference(reference(verse));
          prepare(true);
        };
        return row;
      }),
    );
  }
  document
    .querySelectorAll(".native-verse")
    .forEach((row) =>
      row.classList.toggle(
        "selected",
        Number(row.dataset.verse) >= start && Number(row.dataset.verse) <= end,
      ),
    );
  requestAnimationFrame(() =>
    revealInList(
      $("native-verses"),
      $("native-verses").querySelector(".selected"),
    ),
  );
  if (result.selection && document.body.dataset.page === "bible") prepare();
  renderBooks(previousBook !== chapter.book);
}

async function loadBooks() {
  try {
    const code =
      $("bible-version").value || model.state?.bible_version || "ARA";
    if (version !== code) {
      version = code;
      books = await request("books?version=" + encodeURIComponent(code));
      renderBooks();
    }
    const context = model.state?.bible_context;
    const query =
      $("bible-query").value ||
      `${context?.book || books[0].name} ${context?.chapter || 1}:${context?.start || 1}`;
    selectBibleReference(query);
  } catch (error) {
    version = "";
    report(error);
  }
}

export function setupNativeBible(selectionPanel, lyricsPanel) {
  panel = node("div", undefined, "native-bible-navigation");
  const search = iconButton(t("bible_search"), "search", () => {
    $("native-bible-search").showModal();
    selectBibleReference("");
    $("bible-query").focus();
  });
  search.id = "native-bible-search-toggle";
  const filter = iconButton(t("all_books"), "filter", () => {
    panel.dataset.testament =
      panel.dataset.testament === "new"
        ? "all"
        : panel.dataset.testament === "old"
          ? "new"
          : "old";
  });
  const layout = iconButton(t("cards_layout"), "grid", () =>
    panel.classList.toggle("books-cards"),
  );
  const tools = node("div", undefined, "native-toolbar");
  tools.append(search, filter, layout);
  const selectedVersion = node("select");
  selectedVersion.id = "native-version";
  selectedVersion.dataset.tip = "bible_version";
  selectedVersion.setAttribute("aria-label", t("bible_version"));
  selectedVersion.onchange = () => {
    $("bible-version").value = selectedVersion.value;
    $("bible-version").dispatchEvent(new Event("change"));
    loadBooks();
  };
  tree = node("div", undefined, "native-book-tree");
  tree.id = "native-books";
  panel.append(tools, selectedVersion, tree);
  selectionPanel.append(panel);
  const source = $("bible");
  const dialog = node("dialog");
  dialog.id = "native-bible-search";
  dialog.className = "native-bible-search";
  dialog.setAttribute("aria-label", t("bible_search"));
  const close = iconButton(t("close"), "close", () => dialog.close(), "native-search-close");
  dialog.append(close);
  dialog.append(source);
  document.body.append(dialog);
  const footer = source.querySelector(".bible-footer");
  const content = node("div", undefined, "native-bible-content");
  content.id = "native-bible-content";
  const project = $("bible-project");
  project.className = "icon-button";
  project.dataset.tip = "project";
  project.replaceChildren(icon("play"));
  project.setAttribute("aria-label", t("project"));
  const preview = $("bible-preview");
  preview.classList.add("native-extra-tools");
  const actions = node("div", undefined, "native-toolbar");
  actions.append(
    project,
    $("bible-add"),
    iconButton(t("pin"), "pin", () => send({ action: "pin_preview" })),
    preview,
  );
  const heading = node("div", undefined, "native-chapter-heading");
  const previous = iconButton(t("previous_item"), "previous", () =>
    loadChapter(chapter.book, chapter.number - 1),
  );
  previous.id = "native-chapter-previous";
  const next = iconButton(t("next_item"), "next", () =>
    loadChapter(chapter.book, chapter.number + 1),
  );
  next.id = "native-chapter-next";
  const title = node("strong");
  title.id = "native-chapter-title";
  heading.append(previous, title, next);
  const verses = node("div");
  verses.id = "native-verses";
  content.append(actions, heading, footer.querySelector(".check"), verses);
  lyricsPanel.append(content);
  footer.remove();
  const done = button(
    t("select_reference"),
    () => dialog.close(),
    "native-select-reference",
  );
  done.dataset.i18n = "select_reference";
  dialog.append(done);
  const options = new MutationObserver(() => {
    selectedVersion.replaceChildren(
      ...Array.from($("bible-version").options, (option) =>
        option.cloneNode(true),
      ),
    );
    selectedVersion.value = $("bible-version").value;
    loadBooks();
  });
  options.observe($("bible-version"), { childList: true });
  $("bible-version").addEventListener("change", () => {
    selectedVersion.value = $("bible-version").value;
    loadBooks();
  });
  $("bible-merge").addEventListener("change", () => prepare());
  document.addEventListener("control-bible-result", (event) =>
    renderVerses(event.detail),
  );
  document.addEventListener("control-native-page", () => {
    if (document.body.dataset.page === "bible") {
      openBible().then(loadBooks);
    }
  });
}
