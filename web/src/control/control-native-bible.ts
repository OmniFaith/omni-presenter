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
import {
  openBible,
  selectBibleReference,
  type BibleSearchResult,
} from "/control-bible.js";

export interface BookInfo {
  name: string;
  chapters: number;
}

export interface ChapterInfo {
  book: string;
  number: number;
  total: number;
  verses: string[];
}

let books: BookInfo[] = [];
let version = "";
let expanded: string | null = null;
let chapter: ChapterInfo | null = null;
let chapterKey = "";
let start = 1;
let end = 1;
let anchor = 1;
let panel: HTMLElement;
let tree: HTMLElement;

function reference(verse = 1, last = verse): string {
  if (!chapter) return "";
  return `${chapter.book} ${chapter.number}:${verse}${verse === last ? "" : "-" + last}`;
}

async function prepare(project = false): Promise<void> {
  if (!chapter) return;
  const versionSelect = $<HTMLSelectElement>("bible-version");
  const mergeCheck = $<HTMLInputElement>("bible-merge");
  await send({
    action: "bible",
    reference: reference(start, end),
    version: versionSelect ? versionSelect.value : "",
    merge: mergeCheck ? mergeCheck.checked : false,
    project,
    playlist: false,
  });
}

function loadChapter(book: string, number: number): Promise<void> {
  return selectBibleReference(`${book} ${number}:1`).then(() => {
    if (matchMedia("(max-width:860px)").matches) {
      showView("operation", "preview");
    }
  });
}

function renderBooks(follow = false): void {
  if (!tree) return;
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
  if (follow) {
    revealInList(tree, tree.querySelector<HTMLElement>(".native-book > button.active"));
  }
}

function renderVerses(result: BibleSearchResult & { chapter?: ChapterInfo }): void {
  const previousBook = chapter?.book;
  chapter = result.chapter ?? null;
  if (!chapter) return;
  if (previousBook !== chapter.book) expanded = chapter.book;
  const range = result.selection?.reference.match(/:(\d+)(?:[–-](\d+))?$/);
  start = range ? Number(range[1]) : 1;
  end = range ? Number(range[2] || range[1]) : start;
  const titleEl = $("native-chapter-title");
  if (titleEl) titleEl.textContent = chapter.book + " " + chapter.number;
  const prevBtn = $<HTMLButtonElement>("native-chapter-previous");
  if (prevBtn) prevBtn.disabled = chapter.number <= 1;
  const nextBtn = $<HTMLButtonElement>("native-chapter-next");
  if (nextBtn) nextBtn.disabled = chapter.number >= chapter.total;

  const key = JSON.stringify(chapter);
  const versesEl = $("native-verses");
  if (versesEl && key !== chapterKey) {
    chapterKey = key;
    versesEl.replaceChildren(
      ...chapter.verses.map((text, index) => {
        const verse = index + 1;
        const row = button("", () => {}, "native-verse");
        row.dataset.verse = String(verse);
        row.replaceChildren(
          node("span", String(verse), "native-verse-number"),
          node("span", text),
        );
        row.onclick = (event: MouseEvent) => {
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
    .querySelectorAll<HTMLElement>(".native-verse")
    .forEach((row) =>
      row.classList.toggle(
        "selected",
        Number(row.dataset.verse) >= start && Number(row.dataset.verse) <= end,
      ),
    );
  requestAnimationFrame(() => {
    const versesContainer = $("native-verses");
    if (versesContainer) {
      revealInList(
        versesContainer,
        versesContainer.querySelector<HTMLElement>(".selected"),
      );
    }
  });
  if (result.selection && document.body.dataset.page === "bible") prepare();
  renderBooks(previousBook !== chapter.book);
}

async function loadBooks(): Promise<void> {
  try {
    const versionSelect = $<HTMLSelectElement>("bible-version");
    const code =
      versionSelect?.value || model.state?.bible_version || "ARA";
    if (version !== code) {
      version = code;
      books = await request<BookInfo[]>("books?version=" + encodeURIComponent(code));
      renderBooks();
    }
    const context = model.state?.bible_context;
    const queryInput = $<HTMLInputElement>("bible-query");
    const defaultBook = books[0] ? books[0].name : "Gênesis";
    const query =
      queryInput?.value ||
      `${context?.book || defaultBook} ${context?.chapter || 1}:${context?.start || 1}`;
    selectBibleReference(query);
  } catch (error) {
    version = "";
    report(error);
  }
}

export function setupNativeBible(
  selectionPanel: HTMLElement,
  lyricsPanel: HTMLElement,
): void {
  panel = node("div", undefined, "native-bible-navigation");
  const search = iconButton(t("bible_search"), "search", () => {
    $<HTMLDialogElement>("native-bible-search")?.showModal();
    selectBibleReference("");
    $<HTMLInputElement>("bible-query")?.focus();
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
    const mainSelect = $<HTMLSelectElement>("bible-version");
    if (mainSelect) {
      mainSelect.value = selectedVersion.value;
      mainSelect.dispatchEvent(new Event("change"));
    }
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
  if (source) dialog.append(source);
  document.body.append(dialog);
  const footer = source?.querySelector(".bible-footer");
  const content = node("div", undefined, "native-bible-content");
  content.id = "native-bible-content";
  const project = $<HTMLButtonElement>("bible-project");
  if (project) {
    project.className = "icon-button";
    project.dataset.tip = "project";
    project.replaceChildren(icon("play"));
    project.setAttribute("aria-label", t("project"));
  }
  const preview = $("bible-preview");
  if (preview) preview.classList.add("native-extra-tools");
  const actions = node("div", undefined, "native-toolbar");
  if (project) actions.append(project);
  const bibleAdd = $("bible-add");
  if (bibleAdd) actions.append(bibleAdd);
  actions.append(
    iconButton(t("pin"), "pin", () => send({ action: "pin_preview" })),
  );
  if (preview) actions.append(preview);
  const heading = node("div", undefined, "native-chapter-heading");
  const previous = iconButton(t("previous_item"), "previous", () => {
    if (chapter) loadChapter(chapter.book, chapter.number - 1);
  });
  previous.id = "native-chapter-previous";
  const next = iconButton(t("next_item"), "next", () => {
    if (chapter) loadChapter(chapter.book, chapter.number + 1);
  });
  next.id = "native-chapter-next";
  const title = node("strong");
  title.id = "native-chapter-title";
  heading.append(previous, title, next);
  const verses = node("div");
  verses.id = "native-verses";
  const checkEl = footer?.querySelector(".check");
  content.append(actions, heading);
  if (checkEl) content.append(checkEl);
  content.append(verses);
  lyricsPanel.append(content);
  if (footer) footer.remove();
  const done = button(
    t("select_reference"),
    () => dialog.close(),
    "native-select-reference",
  );
  done.dataset.i18n = "select_reference";
  dialog.append(done);
  const bibleVersionSelect = $<HTMLSelectElement>("bible-version");
  if (bibleVersionSelect) {
    const observer = new MutationObserver(() => {
      selectedVersion.replaceChildren(
        ...Array.from(bibleVersionSelect.options, (option) =>
          option.cloneNode(true),
        ),
      );
      selectedVersion.value = bibleVersionSelect.value;
      loadBooks();
    });
    observer.observe(bibleVersionSelect, { childList: true });
    bibleVersionSelect.addEventListener("change", () => {
      selectedVersion.value = bibleVersionSelect.value;
      loadBooks();
    });
  }
  $("bible-merge")?.addEventListener("change", () => prepare());
  document.addEventListener("control-bible-result", (event: Event) =>
    renderVerses((event as CustomEvent).detail),
  );
  document.addEventListener("control-native-page", () => {
    if (document.body.dataset.page === "bible") {
      openBible().then(loadBooks);
    }
  });
}

