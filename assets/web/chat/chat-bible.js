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
    const ref = $("bible-reference");
    if (ref) {
        ref.value = query;
        ref.focus();
        ref.setSelectionRange(query.length, query.length);
    }
    searchBible();
}
async function searchBible(autocomplete = false) {
    const request = ++bibleRequest;
    resetBibleSelection();
    const resultsEl = $("bible-results");
    if (resultsEl)
        resultsEl.setAttribute("aria-busy", "true");
    try {
        const versionVal = $("bible-version")?.value || "ARA";
        const refVal = $("bible-reference")?.value || "";
        const params = new URLSearchParams({
            version: versionVal,
            q: refVal,
        });
        const result = await api("/bible/search?" + params);
        const dialog = $("bible-dialog");
        if (request !== bibleRequest || !dialog?.open)
            return;
        const refInput = $("bible-reference");
        if (autocomplete &&
            result.completion &&
            result.completion !== refInput?.value &&
            !result.selection &&
            refInput) {
            refInput.value = result.completion;
            refInput.setSelectionRange(result.completion.length, result.completion.length);
            return searchBible(true);
        }
        const stages = {
            book: "Livro",
            chapter: "Capítulo",
            verse: "Versículo",
            text: "Resultados no texto",
        };
        const stageEl = $("bible-stage");
        if (stageEl) {
            stageEl.textContent =
                interfaceTexts["chat.bible_stage." + result.stage] ||
                    stages[result.stage] ||
                    result.stage;
        }
        const grid = node("div", undefined, "reference-grid " + result.stage);
        bibleChoiceIndex = 0;
        result.choices.forEach((choice, index) => {
            const btn = node("button", choice.label);
            btn.type = "button";
            if (choice.text)
                btn.append(node("small", choice.text));
            if (choice.query === result.selection?.reference)
                bibleChoiceIndex = index;
            btn.onclick = () => chooseBibleReference(choice.query);
            grid.append(btn);
        });
        resultsEl?.replaceChildren(grid);
        if (!result.choices.length && resultsEl) {
            resultsEl.append(node("p", interfaceTexts["chat.bible_empty"] ||
                "Nenhuma passagem encontrada."));
        }
        bibleSelection = result.selection ?? null;
        if (bibleSelection) {
            const prevEl = $("bible-preview");
            if (prevEl) {
                prevEl.replaceChildren(node("strong", bibleSelection.reference + " · " + bibleSelection.version), node("pre", bibleSelection.text));
                prevEl.hidden = false;
            }
            $("bible-send").disabled = bibleSending;
        }
        highlightBibleChoice(false);
    }
    catch (error) {
        if (request === bibleRequest) {
            $("bible-error").textContent = error.message;
        }
    }
    finally {
        if (request === bibleRequest) {
            resultsEl?.removeAttribute("aria-busy");
        }
    }
}
function highlightBibleChoice(scroll) {
    const choices = $("bible-results")?.querySelectorAll("button");
    if (!choices)
        return;
    choices.forEach((choice, index) => {
        choice.classList.toggle("selected", index === bibleChoiceIndex);
        choice.setAttribute("aria-pressed", String(index === bibleChoiceIndex));
    });
    if (scroll && choices[bibleChoiceIndex]) {
        choices[bibleChoiceIndex].scrollIntoView({ block: "nearest" });
    }
}
const bibleBtn = $("bible");
if (bibleBtn) {
    bibleBtn.onclick = async () => {
        bibleDestination = target();
        resetBibleSelection();
        $("bible-dialog")?.showModal();
        const refInput = $("bible-reference");
        if (refInput) {
            refInput.value = "";
            refInput.focus();
        }
        try {
            const versions = await api("/versions");
            const vSelect = $("bible-version");
            if (vSelect) {
                const previous = vSelect.value || "ARA";
                vSelect.replaceChildren(...versions.map((version) => {
                    const option = node("option", version.id + " · " + version.name);
                    option.value = version.id;
                    return option;
                }));
                if (versions.some((version) => version.id === previous)) {
                    vSelect.value = previous;
                }
            }
            await searchBible();
        }
        catch (error) {
            $("bible-error").textContent = error.message;
        }
    };
}
const bibleRefInput = $("bible-reference");
if (bibleRefInput) {
    bibleRefInput.oninput = (event) => {
        clearTimeout(bibleTimer);
        ++bibleRequest;
        resetBibleSelection();
        $("bible-results")?.replaceChildren();
        const inputEv = event;
        bibleTimer = setTimeout(() => searchBible(inputEv.inputType === "insertText"), 120);
    };
    bibleRefInput.onkeydown = (event) => {
        const choices = $("bible-results")?.querySelectorAll("button");
        if (!choices || !choices.length) {
            if (event.key === "Enter" && !bibleSelection)
                event.preventDefault();
            return;
        }
        const grid = $("bible-results")?.querySelector(".reference-grid");
        const columns = grid
            ? getComputedStyle(grid).gridTemplateColumns.split(" ").length
            : 1;
        const moves = {
            ArrowDown: columns,
            ArrowUp: -columns,
            ArrowLeft: -1,
            ArrowRight: 1,
        };
        if (event.key in moves) {
            event.preventDefault();
            bibleChoiceIndex = Math.max(0, Math.min(choices.length - 1, bibleChoiceIndex + moves[event.key]));
            highlightBibleChoice(true);
        }
        else if (event.key === "Enter" && !bibleSelection) {
            event.preventDefault();
            choices[bibleChoiceIndex]?.click();
        }
    };
}
const bibleVersionSelect = $("bible-version");
if (bibleVersionSelect) {
    bibleVersionSelect.onchange = () => searchBible();
}
const bibleDialog = $("bible-dialog");
if (bibleDialog) {
    bibleDialog.onclose = () => {
        clearTimeout(bibleTimer);
        ++bibleRequest;
        resetBibleSelection();
    };
}
const bibleForm = $("bible-form");
if (bibleForm) {
    bibleForm.onsubmit = async (event) => {
        event.preventDefault();
        if (!bibleSelection || bibleSending)
            return;
        bibleSending = true;
        $("bible-send").disabled = true;
        try {
            const mergeCheck = $("bible-merge");
            await send({
                kind: "bible",
                reference: bibleSelection.reference,
                version: bibleSelection.version,
                merge: mergeCheck ? mergeCheck.checked : false,
            }, bibleDestination);
            bibleDialog?.close();
        }
        catch (error) {
            $("bible-error").textContent = error.message;
        }
        finally {
            bibleSending = false;
            const sendBtn = $("bible-send");
            if (sendBtn)
                sendBtn.disabled = !bibleSelection;
        }
    };
}