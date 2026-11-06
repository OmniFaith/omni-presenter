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

let selection,
  sequence = 0,
  timer,
  choiceIndex = 0;
export function selectBibleReference(query, complete = false) {
  clearTimeout(timer);
  $("bible-query").value = query;
  return searchBible().then(() => {
    if (complete && selection && $("native-bible-search")?.open) {
      $("native-bible-search").close();
      showView("operation", "preview");
    }
  });
}
export async function openBible(refresh = false) {
  if (refresh || !$("bible-version").options.length) {
    const selected =
      $("bible-version").value || model.state?.bible_version || "ARA";
    try {
      const versions = await request("versions");
      $("bible-version").replaceChildren(
        ...versions.map((version) => {
          const option = node("option", version.id + " · " + version.name);
          option.value = version.id;
          return option;
        }),
      );
      $("bible-version").value = selected;
    } catch (error) {
      report(error);
      return;
    }
  }
  searchBible();
  $("bible-query").focus();
}

function resetSelection() {
  selection = null;
  $("bible-passage").hidden = true;
  ["bible-preview", "bible-add", "bible-project"].forEach(
    (id) => ($(id).disabled = true),
  );
}

async function searchBible(autocomplete = false) {
  const current = ++sequence;
  resetSelection();
  try {
    const result = await request(
      "bible?" +
        new URLSearchParams({
          version: $("bible-version").value,
          q: $("bible-query").value,
        }),
    );
    if (current !== sequence) return;
    if (
      autocomplete &&
      result.completion &&
      result.completion !== $("bible-query").value &&
      !result.selection
    ) {
      $("bible-query").value = result.completion;
      return searchBible(true);
    }
    $("bible-stage").textContent =
      t("chat.bible_stage." + result.stage) ||
      model.texts["chat.bible_stage." + result.stage] ||
      result.stage;
    $("bible-choices").className = "bible-choices " + result.stage;
    choiceIndex = 0;
    $("bible-choices").replaceChildren(
      ...result.choices.map((choice, index) => {
        const item = button(choice.label, () => {
          selectBibleReference(choice.query, true);
        });
        if (choice.text) item.append(node("small", choice.text));
        if (choice.query === result.selection?.reference) choiceIndex = index;
        return item;
      }),
    );
    highlight();
    selection = result.selection;
    if (selection) {
      $("bible-passage").replaceChildren(
        node("strong", selection.reference + " · " + selection.version),
        node("div", selection.text),
      );
      $("bible-passage").hidden = false;
      ["bible-preview", "bible-add", "bible-project"].forEach(
        (id) => ($(id).disabled = false),
      );
    }
    if (!result.choices.length)
      $("bible-choices").append(node("p", t("empty_bible"), "empty"));
    document.dispatchEvent(
      new CustomEvent("control-bible-result", { detail: result }),
    );
    if (autocomplete && result.auto_select && $("native-bible-search")?.open) {
      $("native-bible-search").close();
      showView("operation", "preview");
    }
  } catch (error) {
    if (current === sequence) report(error);
  }
}

function highlight(scroll = false) {
  const choices = $("bible-choices").querySelectorAll("button");
  choices.forEach((item, index) =>
    item.classList.toggle("active", index === choiceIndex),
  );
    if (scroll) choices[choiceIndex]?.scrollIntoView({ block: "nearest" });
}

export function setupBible() {
  $("bible-query").oninput = (event) => {
    ++sequence;
    resetSelection();
    $("bible-choices").replaceChildren();
    clearTimeout(timer);
    timer = setTimeout(() => searchBible(event.inputType === "insertText"), 90);
  };
  $("bible-query").onkeydown = (event) => {
    if ((event.key === "Enter" || event.key === "Tab") && !event.shiftKey && selection && $("native-bible-search")?.open) {
      event.preventDefault();
      $("native-bible-search").close();
      showView("operation", "preview");
      return;
    }
    const choices = $("bible-choices").querySelectorAll("button");
    if (!choices.length) return;
    const columns = getComputedStyle(
      $("bible-choices"),
    ).gridTemplateColumns.split(" ").length;
    const move = {
      ArrowLeft: -1,
      ArrowRight: 1,
      ArrowUp: -columns,
      ArrowDown: columns,
    }[event.key];
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
      choices[choiceIndex].click();
    }
  };
  $("bible-version").onchange = () => searchBible();
  $("bible-reset").onclick = () => {
    $("bible-query").value = "";
    searchBible();
  };
  for (const [id, project, playlist] of [
    ["bible-preview", false, false],
    ["bible-add", false, true],
    ["bible-project", true, false],
  ]) {
    $(id).onclick = async () => {
      if (!selection) return;
      if (
        await send(
          {
            action: "bible",
            reference: selection.reference,
            version: selection.version,
            merge: $("bible-merge").checked,
            project,
            playlist,
          },
          $(id),
        )
      ) {
        if (playlist) toast(t("added"));
        else showView("operation", project ? "live" : "preview");
      }
    };
  }
}
