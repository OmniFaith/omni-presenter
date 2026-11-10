declare function status(text: string): void;

interface EmojiItem {
  name: string;
  value: string;
}

interface EmojiCategory {
  name: string;
  icon: string;
  emojis: EmojiItem[];
}

async function openEmojiPicker(
  messageId: string | number | null,
  anchor: DOMRect,
): Promise<void> {
  try {
    if (!state.emojiCategories.length) {
      state.emojiCategories = await api<EmojiCategory[]>("/emojis");
    }
    state.reactionTarget = messageId;
    if (!$("emoji-filter")) {
      const heading = node("div", undefined, "emoji-heading");
      heading.append(node("strong", "Emojis"));
      const close = node("button", "×");
      close.title = "Fechar emojis";
      close.onclick = () => {
        $("emojis").hidden = true;
      };
      heading.append(close);
      const search = node("input");
      search.id = "emoji-filter";
      search.placeholder = "Pesquisar emoji…";
      search.type = "search";
      search.oninput = () => renderEmojiGrid();
      const quick = node("div", undefined, "emoji-quick");
      quick.id = "emoji-quick";
      const tabs = node("div", undefined, "emoji-tabs");
      tabs.id = "emoji-tabs";
      const grid = node("div", undefined, "emoji-results");
      grid.id = "emoji-results";
      $("emojis").replaceChildren(heading, quick, search, tabs, grid);
    }
    const strongEl = $("emojis").querySelector("strong");
    if (strongEl) {
      strongEl.textContent =
        messageId === null ? "Emojis" : "Reagir à mensagem";
    }
    $("emojis").classList.toggle("reaction-mode", messageId !== null);
    $("emoji-quick").hidden = messageId === null;
    $("emoji-quick").replaceChildren(
      ...["👍", "❤️", "😂", "🙏", "🙌", "🎉", "👀", "🔥"].map((emoji) => {
        const btn = node("button", emoji);
        btn.title = "Reagir com " + emoji;
        btn.onclick = () => pickEmoji(emoji);
        return btn;
      }),
    );
    const filterInput = $<HTMLInputElement>("emoji-filter");
    if (filterInput) filterInput.value = "";
    renderEmojiGrid();
    $("emojis").hidden = false;
    const width = Math.min(350, innerWidth - 20);
    const height = Math.min(390, innerHeight - 30);
    $("emojis").style.width = width + "px";
    $("emojis").style.left =
      Math.max(10, Math.min(innerWidth - width - 10, anchor.right - width)) +
      "px";
    $("emojis").style.top =
      Math.max(
        10,
        Math.min(innerHeight - height - 10, anchor.top - height - 6),
      ) + "px";
    $("emojis").style.right = "auto";
    $("emojis").style.bottom = "auto";
  } catch (error) {
    status((error as Error).message);
  }
}

function renderEmojiGrid(): void {
  const categories = state.emojiCategories as EmojiCategory[];
  $("emoji-tabs").replaceChildren(
    ...categories.map((category, index) => {
      const btn = node(
        "button",
        category.icon,
        index === state.emojiCategory ? "selected" : "",
      );
      btn.title = category.name;
      btn.setAttribute("aria-label", category.name);
      btn.onclick = () => {
        state.emojiCategory = index;
        const filterEl = $<HTMLInputElement>("emoji-filter");
        if (filterEl) filterEl.value = "";
        renderEmojiGrid();
      };
      return btn;
    }),
  );
  const fold = (s: string) =>
    s
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, "")
      .toLowerCase();
  const filterVal = $<HTMLInputElement>("emoji-filter")?.value || "";
  const query = fold(filterVal);
  const results: HTMLElement[] = [];
  for (const [index, category] of categories.entries()) {
    if (!query && index !== state.emojiCategory) continue;
    const emojis = category.emojis.filter(
      (e) => !query || fold(e.name).includes(query) || e.value.includes(query),
    );
    if (!emojis.length) continue;
    results.push(node("small", category.name));
    const grid = node("div", undefined, "emoji-grid");
    for (const emoji of emojis) {
      const btn = node("button", emoji.value);
      btn.title = emoji.name;
      btn.setAttribute("aria-label", emoji.name);
      btn.onclick = () => {
        pickEmoji(emoji.value);
      };
      grid.append(btn);
    }
    results.push(grid);
  }
  $("emoji-results").replaceChildren(...results);
}

function pickEmoji(value: string): void {
  const id = state.reactionTarget as string | number | null;
  $("emojis").hidden = true;
  if (id !== null) {
    (window as unknown as { toggleReaction: (id: string | number, val: string) => void }).toggleReaction(id, value);
  } else {
    const input = $<HTMLTextAreaElement>("input");
    if (input) {
      input.value += value;
      composer();
      input.focus();
    }
  }
}

const emojiBtn = $("emoji");
if (emojiBtn) {
  emojiBtn.onclick = (event: MouseEvent) => {
    event.stopPropagation();
    if (!$("emojis").hidden && state.reactionTarget === null) {
      $("emojis").hidden = true;
    } else {
      openEmojiPicker(null, emojiBtn.getBoundingClientRect());
    }
  };
}

document.addEventListener("click", (event: MouseEvent) => {
  const target = event.target as HTMLElement | null;
  if (
    !$("emojis").hidden &&
    !$("emojis").contains(target) &&
    !target?.closest("#emoji") &&
    !target?.closest(".message-tools")
  ) {
    $("emojis").hidden = true;
  }
});

document.addEventListener("keydown", (event: KeyboardEvent) => {
  if (event.key === "Escape") $("emojis").hidden = true;
});

const attachBtn = $("attach");
if (attachBtn) {
  attachBtn.onclick = () => $<HTMLInputElement>("file")?.click();
}

