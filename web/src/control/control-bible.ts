import {
  $,
  model,
  t,
  node,
  button,
  request,
  report,
  send,
  showView,
  toast,
} from "/control-api.js";

export interface BibleChoice {
  label: string;
  query: string;
  text?: string;
}

export interface BibleSelection {
  reference: string;
  version: string;
  text: string;
}

export interface BibleSearchResult {
  stage: string;
  choices: BibleChoice[];
  selection?: BibleSelection | null;
  completion?: string;
  auto_select?: boolean;
}

export interface BibleVersionItem {
  id: string;
  name: string;
}

let selection: BibleSelection | null = null;
let sequence = 0;
let timer: ReturnType<typeof setTimeout> | undefined;
let choiceIndex = 0;

export function selectBibleReference(
  query: string,
  complete = false,
): Promise<void> {
  clearTimeout(timer);
  const queryInput = $<HTMLInputElement>("bible-query");
  if (queryInput) queryInput.value = query;
  return searchBible().then(() => {
    const dialog = $<HTMLDialogElement>("native-bible-search");
    if (complete && selection && dialog?.open) {
      dialog.close();
      showView("operation", "preview");
    }
  });
}

export async function openBible(refresh = false): Promise<void> {
  const versionSelect = $<HTMLSelectElement>("bible-version");
  if (!versionSelect) return;
  if (refresh || !versionSelect.options.length) {
    const selected =
      versionSelect.value || model.state?.bible_version || "ARA";
    try {
      const versions = await request<BibleVersionItem[]>("versions");
      versionSelect.replaceChildren(
        ...versions.map((version) => {
          const option = node("option", version.id + " · " + version.name);
          option.value = version.id;
          return option;
        }),
      );
      versionSelect.value = selected;
    } catch (error) {
      report(error);
      return;
    }
  }
  searchBible();
  const queryInput = $<HTMLInputElement>("bible-query");
  if (queryInput) queryInput.focus();
}

function resetSelection(): void {
  selection = null;
  const passage = $("bible-passage");
  if (passage) passage.hidden = true;
  ["bible-preview", "bible-add", "bible-project"].forEach((id) => {
    const el = $<HTMLButtonElement>(id);
    if (el) el.disabled = true;
  });
}

async function searchBible(autocomplete = false): Promise<void> {
  const current = ++sequence;
  resetSelection();
  const queryInput = $<HTMLInputElement>("bible-query");
  const versionSelect = $<HTMLSelectElement>("bible-version");
  if (!queryInput || !versionSelect) return;

  try {
    const result = await request<BibleSearchResult>(
      "bible?" +
        new URLSearchParams({
          version: versionSelect.value,
          q: queryInput.value,
        }),
    );
    if (current !== sequence) return;
    if (
      autocomplete &&
      result.completion &&
      result.completion !== queryInput.value &&
      !result.selection
    ) {
      queryInput.value = result.completion;
      return searchBible(true);
    }
    const stageEl = $("bible-stage");
    if (stageEl) {
      stageEl.textContent =
        t("chat.bible_stage." + result.stage) ||
        model.texts["chat.bible_stage." + result.stage] ||
        result.stage;
    }
    const choicesEl = $("bible-choices");
    if (choicesEl) {
      choicesEl.className = "bible-choices " + result.stage;
      choiceIndex = 0;
      choicesEl.replaceChildren(
        ...result.choices.map((choice, index) => {
          const item = button(choice.label, () => {
            selectBibleReference(choice.query, true);
          });
          if (choice.text) item.append(node("small", choice.text));
          if (choice.query === result.selection?.reference)
            choiceIndex = index;
          return item;
        }),
      );
    }
    highlight();
    selection = result.selection ?? null;
    if (selection) {
      const passage = $("bible-passage");
      if (passage) {
        passage.replaceChildren(
          node("strong", selection.reference + " · " + selection.version),
          node("div", selection.text),
        );
        passage.hidden = false;
      }
      ["bible-preview", "bible-add", "bible-project"].forEach((id) => {
        const el = $<HTMLButtonElement>(id);
        if (el) el.disabled = false;
      });
    }
    if (choicesEl && !result.choices.length) {
      choicesEl.append(node("p", t("empty_bible"), "empty"));
    }
    document.dispatchEvent(
      new CustomEvent("control-bible-result", { detail: result }),
    );
    const dialog = $<HTMLDialogElement>("native-bible-search");
    if (autocomplete && result.auto_select && dialog?.open) {
      dialog.close();
      showView("operation", "preview");
    }
  } catch (error) {
    if (current === sequence) report(error);
  }
}

function highlight(scroll = false): void {
  const choices = $("bible-choices")?.querySelectorAll<HTMLButtonElement>("button");
  if (!choices) return;
  choices.forEach((item, index) =>
    item.classList.toggle("active", index === choiceIndex),
  );
  if (scroll && choices[choiceIndex]) {
    choices[choiceIndex].scrollIntoView({ block: "nearest" });
  }
}

export function setupBible(): void {
  const queryInput = $<HTMLInputElement>("bible-query");
  if (queryInput) {
    queryInput.oninput = (event: Event) => {
      ++sequence;
      resetSelection();
      $("bible-choices")?.replaceChildren();
      clearTimeout(timer);
      const inputEvent = event as InputEvent;
      timer = setTimeout(
        () => searchBible(inputEvent.inputType === "insertText"),
        90,
      );
    };
    queryInput.onkeydown = (event: KeyboardEvent) => {
      const dialog = $<HTMLDialogElement>("native-bible-search");
      if (
        (event.key === "Enter" || event.key === "Tab") &&
        !event.shiftKey &&
        selection &&
        dialog?.open
      ) {
        event.preventDefault();
        dialog.close();
        showView("operation", "preview");
        return;
      }
      const choices = $("bible-choices")?.querySelectorAll<HTMLButtonElement>("button");
      if (!choices || !choices.length) return;
      const choicesEl = $("bible-choices");
      const columns = choicesEl
        ? getComputedStyle(choicesEl).gridTemplateColumns.split(" ").length
        : 1;
      const move = ({
        ArrowLeft: -1,
        ArrowRight: 1,
        ArrowUp: -columns,
        ArrowDown: columns,
      } as Record<string, number>)[event.key];
      if (move) {
        event.preventDefault();
        choiceIndex = Math.max(
          0,
          Math.min(choices.length - 1, choiceIndex + move),
        );
        highlight(true);
      }
      if (event.key === "Enter" || (event.key === "Tab" && !event.shiftKey)) {
        event.preventDefault();
        choices[choiceIndex]?.click();
      }
    };
  }
  const versionSelect = $<HTMLSelectElement>("bible-version");
  if (versionSelect) {
    versionSelect.onchange = () => searchBible();
  }
  const resetBtn = $("bible-reset");
  if (resetBtn) {
    resetBtn.onclick = () => {
      if (queryInput) queryInput.value = "";
      searchBible();
    };
  }
  for (const [id, project, playlist] of [
    ["bible-preview", false, false],
    ["bible-add", false, true],
    ["bible-project", true, false],
  ] as const) {
    const btn = $<HTMLButtonElement>(id);
    if (btn) {
      btn.onclick = async () => {
        if (!selection) return;
        const mergeCheck = $<HTMLInputElement>("bible-merge");
        const ok = await send(
          {
            action: "bible",
            reference: selection.reference,
            version: selection.version,
            merge: mergeCheck ? mergeCheck.checked : false,
            project,
            playlist,
          },
          btn,
        );
        if (ok) {
          if (playlist) toast(t("added"));
          else showView("operation", project ? "live" : "preview");
        }
      };
    }
  }
}

