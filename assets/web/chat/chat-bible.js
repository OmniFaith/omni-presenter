"use strict";

let bibleRequest = 0;
let bibleTimer;
let bibleSelection = null;
let bibleDestination;
let bibleSending = false;
let bibleChoiceIndex = 0;

function resetBibleSelection() {
  bibleSelection = null;
  $("bible-send").disabled = true;
  $("bible-preview").hidden = true;
  $("bible-error").textContent = "";
}

function chooseBibleReference(query) {
  $("bible-reference").value = query;
  $("bible-reference").focus();
  $("bible-reference").setSelectionRange(query.length, query.length);
  searchBible();
}

async function searchBible(autocomplete = false) {
  const request = ++bibleRequest;
  resetBibleSelection();
  $("bible-results").setAttribute("aria-busy", "true");
  try {
    const params = new URLSearchParams({ version: $("bible-version").value, q: $("bible-reference").value });
    const result = await api("/bible/search?" + params);
    if (request !== bibleRequest || !$("bible-dialog").open) return;
    if (autocomplete && result.completion && result.completion !== $("bible-reference").value && !result.selection) {
      $("bible-reference").value = result.completion;
      $("bible-reference").setSelectionRange(result.completion.length, result.completion.length);
      return searchBible(true);
    }
    const stages = { book: "Livro", chapter: "Capítulo", verse: "Versículo", text: "Resultados no texto" };
    $("bible-stage").textContent = interfaceTexts["chat.bible_stage." + result.stage] || stages[result.stage];
    const grid = node("div", undefined, "reference-grid " + result.stage);
    bibleChoiceIndex = 0;
    result.choices.forEach((choice, index) => {
      const button = node("button", choice.label);
      button.type = "button";
      if (choice.text) button.append(node("small", choice.text));
      if (choice.query === result.selection?.reference) bibleChoiceIndex = index;
      button.onclick = () => chooseBibleReference(choice.query);
      grid.append(button);
    });
    $("bible-results").replaceChildren(grid);
    if (!result.choices.length) $("bible-results").append(node("p", interfaceTexts["chat.bible_empty"] || "Nenhuma passagem encontrada."));
    bibleSelection = result.selection;
    if (bibleSelection) {
      $("bible-preview").replaceChildren(
        node("strong", bibleSelection.reference + " · " + bibleSelection.version),
        node("pre", bibleSelection.text),
      );
      $("bible-preview").hidden = false;
      $("bible-send").disabled = bibleSending;
    }
    highlightBibleChoice(false);
  } catch (error) {
    if (request === bibleRequest) $("bible-error").textContent = error.message;
  } finally {
    if (request === bibleRequest) $("bible-results").removeAttribute("aria-busy");
  }
}

function highlightBibleChoice(scroll) {
  const choices = $("bible-results").querySelectorAll("button");
  choices.forEach((choice, index) => {
    choice.classList.toggle("selected", index === bibleChoiceIndex);
    choice.setAttribute("aria-pressed", String(index === bibleChoiceIndex));
  });
  if (scroll) choices[bibleChoiceIndex]?.scrollIntoView({ block: "nearest" });
}

$("bible").onclick = async () => {
  bibleDestination = target();
  resetBibleSelection();
  $("bible-dialog").showModal();
  $("bible-reference").value = "";
  $("bible-reference").focus();
  try {
    const versions = await api("/versions");
    const previous = $("bible-version").value || "ARA";
    $("bible-version").replaceChildren(...versions.map((version) => {
      const option = node("option", version.id + " · " + version.name);
      option.value = version.id;
      return option;
    }));
    if (versions.some((version) => version.id === previous)) $("bible-version").value = previous;
    await searchBible();
  } catch (error) {
    $("bible-error").textContent = error.message;
  }
};
$("bible-reference").oninput = (event) => {
  clearTimeout(bibleTimer);
  ++bibleRequest;
  resetBibleSelection();
  $("bible-results").replaceChildren();
  bibleTimer = setTimeout(() => searchBible(event.inputType === "insertText"), 120);
};
$("bible-reference").onkeydown = (event) => {
  const choices = $("bible-results").querySelectorAll("button");
  if (!choices.length) {
    if (event.key === "Enter" && !bibleSelection) event.preventDefault();
    return;
  }
  const grid = $("bible-results").querySelector(".reference-grid");
  const columns = getComputedStyle(grid).gridTemplateColumns.split(" ").length;
  const moves = { ArrowDown: columns, ArrowUp: -columns, ArrowLeft: -1, ArrowRight: 1 };
  if (event.key in moves) {
    event.preventDefault();
    bibleChoiceIndex = Math.max(0, Math.min(choices.length - 1, bibleChoiceIndex + moves[event.key]));
    highlightBibleChoice(true);
  } else if (event.key === "Enter" && !bibleSelection) {
    event.preventDefault();
    choices[bibleChoiceIndex].click();
  }
};
$("bible-version").onchange = () => searchBible();
$("bible-dialog").onclose = () => {
  clearTimeout(bibleTimer);
  ++bibleRequest;
  resetBibleSelection();
};
$("bible-form").onsubmit = async (event) => {
  event.preventDefault();
  if (!bibleSelection || bibleSending) return;
  bibleSending = true;
  $("bible-send").disabled = true;
  try {
    await send({ kind: "bible", reference: bibleSelection.reference, version: bibleSelection.version, merge: $("bible-merge").checked }, bibleDestination);
    $("bible-dialog").close();
  } catch (error) {
    $("bible-error").textContent = error.message;
  } finally {
    bibleSending = false;
    $("bible-send").disabled = !bibleSelection;
  }
};
