const chatGlyphs = {
    chat: "\u{E003}",
    book: "\u{E002}",
    bible: "\u{E002}",
    song: "\u{E001}",
    music: "\u{E001}",
    playlist: "\u{E2F8}",
    play: "\u{E3D0}",
    pause: "\u{E39E}",
    stop: "\u{E46C}",
    trash: "\u{E4A6}",
    smile: "\u{E436}",
    shortcut: "\u{E2A8}",
    pin: "\u{E2A8}",
    link: "\u{E2E6}",
    download: "\u{E20C}",
    file: "\u{E230}",
    image: "\u{E2CA}",
    video: "\u{E740}",
    bell: "\u{E0CE}",
    notify: "\u{E0CE}",
    plus: "\u{E3D4}",
    attach: "\u{E3D4}",
    menu: "\u{E2F0}",
    list: "\u{E2F0}",
    search: "\u{E30C}",
    record: "\u{E326}",
    mic: "\u{E326}",
    send: "\u{E396}",
    close: "\u{E4F6}",
    edit: "\u{E3B4}",
    more: "\u{EAE2}",
};
function icon(name) {
    const glyph = document.createElement("span");
    glyph.className = "native-icon";
    glyph.dataset.nativeIcon = name;
    glyph.textContent = chatGlyphs[name] || name;
    glyph.setAttribute("aria-hidden", "true");
    return glyph;
}
function paintIcons(root = document) {
    root
        .querySelectorAll("[data-icon]")
        .forEach((element) => {
        const name = element.dataset.icon;
        if (name)
            element.replaceChildren(icon(name));
    });
}
async function loadChatIcons() {
    try {
        const response = await fetch("/api/chat/icons");
        if (!response.ok)
            return;
        const remote = (await response.json());
        Object.assign(chatGlyphs, remote);
        document
            .querySelectorAll(".native-icon[data-native-icon]")
            .forEach((element) => {
            const key = element.dataset.nativeIcon;
            const char = key ? chatGlyphs[key] : undefined;
            if (char && element.textContent !== char) {
                element.textContent = char;
            }
        });
    }
    catch {
        /* Fallback remains usable */
    }
}