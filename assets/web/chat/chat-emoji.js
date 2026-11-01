async function openEmojiPicker(messageId, anchor) {
  try {
    if (!state.emojiCategories.length)
      state.emojiCategories = await api("/emojis");
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
      search.oninput = renderEmojiGrid;
      const quick = node("div", undefined, "emoji-quick");
      quick.id = "emoji-quick";
      const tabs = node("div", undefined, "emoji-tabs");
      tabs.id = "emoji-tabs";
      const grid = node("div", undefined, "emoji-results");
      grid.id = "emoji-results";
      $("emojis").replaceChildren(heading, quick, search, tabs, grid);
    }
    $("emojis").querySelector("strong").textContent =
      messageId === null ? "Emojis" : "Reagir à mensagem";
    $("emojis").classList.toggle("reaction-mode", messageId !== null);
    $("emoji-quick").hidden = messageId === null;
    $("emoji-quick").replaceChildren(
      ...["👍", "❤️", "😂", "🙏", "🙌", "🎉", "👀", "🔥"].map((emoji) => {
        const button = node("button", emoji);
        button.title = "Reagir com " + emoji;
        button.onclick = () => pickEmoji(emoji);
        return button;
      }),
    );
    $("emoji-filter").value = "";
    renderEmojiGrid();
    $("emojis").hidden = false;
    const width = Math.min(350, innerWidth - 20),
      height = Math.min(390, innerHeight - 30);
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
    status(error.message);
  }
}
function renderEmojiGrid() {
  $("emoji-tabs").replaceChildren(
    ...state.emojiCategories.map((category, index) => {
      const button = node(
        "button",
        category.icon,
        index === state.emojiCategory ? "selected" : "",
      );
      button.title = category.name;
      button.setAttribute("aria-label", category.name);
      button.onclick = () => {
        state.emojiCategory = index;
        $("emoji-filter").value = "";
        renderEmojiGrid();
      };
      return button;
    }),
  );
  const fold = (s) =>
    s
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, "")
      .toLowerCase();
  const query = fold($("emoji-filter").value);
  const results = [];
  for (const [index, category] of state.emojiCategories.entries()) {
    if (!query && index !== state.emojiCategory) continue;
    const emojis = category.emojis.filter(
      (e) => !query || fold(e.name).includes(query) || e.value.includes(query),
    );
    if (!emojis.length) continue;
    results.push(node("small", category.name));
    const grid = node("div", undefined, "emoji-grid");
    for (const emoji of emojis) {
      const button = node("button", emoji.value);
      button.title = emoji.name;
      button.setAttribute("aria-label", emoji.name);
      button.onclick = () => {
        pickEmoji(emoji.value);
      };
      grid.append(button);
    }
    results.push(grid);
  }
  $("emoji-results").replaceChildren(...results);
}
function pickEmoji(value) {
  const id = state.reactionTarget;
  $("emojis").hidden = true;
  if (id !== null) toggleReaction(id, value);
  else {
    $("input").value += value;
    composer();
    $("input").focus();
  }
}
$("emoji").onclick = (event) => {
  event.stopPropagation();
  if (!$("emojis").hidden && state.reactionTarget === null)
    $("emojis").hidden = true;
  else openEmojiPicker(null, $("emoji").getBoundingClientRect());
};
document.addEventListener("click", (event) => {
  if (
    !$("emojis").hidden &&
    !$("emojis").contains(event.target) &&
    !event.target.closest("#emoji") &&
    !event.target.closest(".message-tools")
  )
    $("emojis").hidden = true;
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") $("emojis").hidden = true;
});
$("attach").onclick = () => $("file").click();
