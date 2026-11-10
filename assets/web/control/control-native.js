import { $, model, t, node, button, iconButton as baseIconButton, showView, send, toast, applyTheme, } from "/control-api.js";
import { icon, paintIcons, loadNativeIcons } from "/control-icons.js";
import { setupDocking, renderDocking, setNativePage, resetNativeLayout, } from "/control-docking.js";
import { setupNativeBible } from "/control-native-bible.js";
let page = "home";
let panelMap;
let cardsOverride = null;
let dockSignature = "";
let outputSignature = "";
const mobile = () => matchMedia("(max-width:860px)").matches;
function iconButton(label, symbol, action, className = "") {
    const control = baseIconButton(label, symbol, action, className);
    if (!Object.keys(model.texts).length)
        control.dataset.tip = label;
    return control;
}
function textButton(key, action) {
    const control = button(t(key), action);
    control.dataset.i18n = key;
    return control;
}
function heading(panel, label, extra = []) {
    const header = panel.querySelector(".panel-heading");
    const title = node("h2", t(label));
    title.dataset.i18n = label;
    const grip = node("span", undefined, "native-header-grip");
    grip.append(icon("grip"));
    header.replaceChildren(title, ...extra, grip);
    return header;
}
function toolbar(...controls) {
    const row = node("div", undefined, "native-toolbar");
    controls.forEach((c) => {
        if (c)
            row.append(c);
    });
    return row;
}
function preparePanels() {
    const createSong = $("song-new");
    const libraryRefresh = $("library-refresh");
    const themeClose = $("themes-close");
    const lyrics = document.querySelector(".slides-panel");
    lyrics.id = "lyrics-panel";
    panelMap = {
        Selection: $("library-panel"),
        Lyrics: lyrics,
        Stage: $("monitor-live"),
        Preview: $("monitor-preview"),
        Themes: $("themes-panel"),
        Playlist: $("playlist-panel"),
        Player: $("player-panel"),
    };
    heading(panelMap.Selection, "native_library");
    const libH2 = panelMap.Selection.querySelector("h2");
    if (libH2)
        libH2.id = "library-heading";
    const search = iconButton(t("search"), "search", () => {
        document.body.dataset.searchOpen =
            document.body.dataset.searchOpen === "true" ? "false" : "true";
        if (document.body.dataset.searchOpen === "true")
            $("library-search")?.focus();
    });
    search.id = "library-search-toggle";
    const folders = iconButton(t("all_folders"), "folder", () => (document.body.dataset.foldersOpen =
        document.body.dataset.foldersOpen === "true" ? "false" : "true"));
    const filter = iconButton(t("filter"), "filter", () => {
        document.body.dataset.foldersOpen = "true";
        $("library-folder")?.focus();
    });
    panelMap.Selection.querySelector(".panel-heading")?.after(toolbar(createSong, search, folders, filter));
    const tabs = document.querySelector(".resource-tabs");
    const keys = {
        song: ["music", "native_songs"],
        media: ["media", "media"],
        text: ["text", "texts"],
        verse: ["verse", "saved_verses"],
        design: ["design", "designs"],
        bible: ["bible", "bible"],
    };
    for (const kind of ["song", "media", "text", "verse", "design", "bible"]) {
        const control = tabs.querySelector(`[data-kind="${kind}"]`);
        if (control) {
            const label = node("span", t(keys[kind][1]));
            label.dataset.i18n = keys[kind][1];
            control.replaceChildren(icon(keys[kind][0]), label);
            tabs.append(control);
        }
    }
    const sourceButtons = lyrics.querySelector(".segmented");
    sourceButtons.classList.add("native-source-control");
    const project = $("project-preview");
    if (project) {
        project.className = "icon-button";
        project.replaceChildren(icon("play"));
        project.title = t("project");
        project.setAttribute("aria-label", t("project"));
    }
    const layout = iconButton(t("cards_layout"), "grid", () => {
        cardsOverride = !(cardsOverride ?? model.state?.lyrics_cards ?? false);
        model.cardsOverride = cardsOverride;
        document.dispatchEvent(new Event("control-slides-layout"));
    });
    layout.id = "lyrics-layout";
    const remove = iconButton(t("remove_playlist"), "trash", async () => {
        const preview = model.state?.preview;
        const items = model.state?.playlist?.filter((item) => item.title === preview?.title && item.subtitle === preview?.subtitle) || [];
        for (const item of items)
            await send({ action: "playlist_remove", id: item.id });
    });
    const pin = iconButton(t("pin"), "pin", () => send({ action: "pin_preview" }));
    const lyricHeader = heading(lyrics, "native_lyrics");
    lyricHeader.after(toolbar(project, layout, $("edit-song"), remove, pin));
    const banner = node("div", undefined, "native-song-heading");
    banner.append(node("strong", "", "native-song-title"), node("span", "", "native-song-artist"));
    lyrics.querySelector(".native-toolbar")?.after(banner);
    lyrics.append(sourceButtons, $("design-controls"));
    heading(panelMap.Stage, "native_live", [$("live-status"), $("live-count")]);
    heading(panelMap.Preview, "native_preview", [$("preview-count")]);
    const previewStage = $("preview-stage");
    previewStage.setAttribute("role", "button");
    previewStage.tabIndex = 0;
    previewStage.title = t("project");
    const previewOutput = node("iframe");
    previewOutput.id = "native-preview-output";
    previewOutput.title = "Visualização";
    previewOutput.tabIndex = -1;
    previewOutput.allow = "autoplay";
    previewStage.append(previewOutput);
    previewStage.onclick = () => send({ action: "project_preview" });
    previewStage.onkeydown = (event) => {
        if (["Enter", " "].includes(event.key)) {
            event.preventDefault();
            event.stopPropagation();
            send({ action: "project_preview" });
        }
    };
    const idle = node("span", "", "native-idle");
    idle.id = "native-live-idle";
    $("live-stage").append(idle);
    const themeToggle = $("theme-live")?.closest("label");
    heading(panelMap.Themes, "themes");
    if (themeToggle) {
        themeToggle.classList.add("native-theme-option");
        panelMap.Themes.append(themeToggle);
    }
    const tools = toolbar(iconButton(t("project"), "play", () => {
        const item = model.state?.playlist?.find((item) => item.selected);
        if (item)
            send({ action: "playlist_select", id: item.id, project: true });
    }), iconButton(t("reorder"), "playlist", (control) => openNativeMenu(control, [
        {
            label: t("sort_title"),
            symbol: "playlist",
            action: async () => {
                if (!model.state?.playlist)
                    return;
                const sorted = [...model.state.playlist].sort((a, b) => a.title.localeCompare(b.title));
                for (let index = 1; index < sorted.length; index++)
                    await send({
                        action: "playlist_move",
                        id: sorted[index].id,
                        target: sorted[index - 1].id,
                        after: true,
                    });
            },
        },
        {
            label: t("reverse"),
            symbol: "refresh",
            action: async () => {
                if (!model.state?.playlist)
                    return;
                const reversed = [...model.state.playlist].reverse();
                for (let index = 1; index < reversed.length; index++)
                    await send({
                        action: "playlist_move",
                        id: reversed[index].id,
                        target: reversed[index - 1].id,
                        after: true,
                    });
            },
        },
        {
            label: t("clear"),
            symbol: "trash",
            action: () => send({ action: "playlist_clear" }),
        },
    ])), iconButton(t("edit_lyrics"), "edit", async () => {
        const item = model.state?.playlist?.find((item) => item.selected);
        if (item && (await send({ action: "playlist_select", id: item.id })))
            $("edit-song").click();
    }), $("playlist-clear"), iconButton(t("save_playlist"), "pin", () => toast(t("saved"))));
    heading(panelMap.Playlist, "native_playlist", [$("playlist-count")]);
    panelMap.Playlist.querySelector(".panel-heading")?.after(tools);
    $("playlist-clear").onclick = () => {
        const sel = model.state?.playlist?.find((item) => item.selected);
        if (sel)
            send({ action: "playlist_remove", id: sel.id });
    };
    const navigation = document.querySelector(".setlist-toolbar");
    if (navigation)
        navigation.classList.add("native-extra-tools");
    const playerTitle = $("player-title");
    heading(panelMap.Player, "player");
    const empty = node("div", undefined, "native-player-empty");
    empty.append(icon("music"), node("strong", t("empty_player")), node("span", t("player_hint"), "native-player-hint"));
    empty.id = "native-player-empty";
    panelMap.Player.append(empty);
    panelMap.Player.querySelector(".player-controls")?.prepend(playerTitle);
    const workspace = document.querySelector(".workspace");
    const root = node("div");
    root.id = "native-layout";
    workspace.append(root);
    const monitorTabs = document.querySelector(".monitor-switch");
    monitorTabs.id = "native-monitor-tabs";
    setupDocking(panelMap);
    $("native-parking").append(monitorTabs, libraryRefresh, themeClose);
    workspace.replaceChildren(root);
    const footer = document.querySelector(".performance-bar");
    const hideControl = $("hide-toggle");
    const stopControl = $("stop");
    const row = node("div", undefined, "native-dock-toolbar");
    row.append(...Array.from(footer.childNodes));
    const transport = row.querySelector(".transport");
    const dockNavigation = transport.querySelector(".navigation");
    const jump = node("input");
    jump.id = "native-slide-jump";
    jump.inputMode = "numeric";
    jump.setAttribute("aria-label", t("go_slide"));
    jump.onchange = () => {
        const index = Number(jump.value) - 1;
        if (Number.isInteger(index) && index >= 0) {
            if (typeof navigator !== "undefined" && navigator.vibrate) {
                try {
                    navigator.vibrate(12);
                }
                catch (_) { }
            }
            send({ action: "live_slide", index });
        }
    };
    jump.dataset.tip = "go_slide";
    jump.onfocus = () => jump.select();
    jump.onclick = () => jump.select();
    jump.onkeydown = (event) => {
        if (event.key === "Enter") {
            event.preventDefault();
            jump.onchange?.(new Event("change"));
        }
    };
    dockNavigation.querySelector('[data-direction="next"]')?.before(jump);
    const progress = node("div", undefined, "native-progress");
    const bar = node("div");
    const fill = node("span");
    fill.id = "native-progress-fill";
    bar.append(fill);
    progress.append(iconButton(t("settings"), "settings", () => $("native-settings")?.showModal()), bar);
    const finalActions = node("div", undefined, "native-dock-final");
    finalActions.append(hideControl, stopControl);
    hideControl.className = "native-dock-action";
    stopControl.className = "native-dock-action danger";
    stopControl.replaceChildren(icon("close"), node("span", t("close_presentation"), "native-button-label"));
    const lastChild = stopControl.lastElementChild;
    if (lastChild)
        lastChild.dataset.i18n = "close_presentation";
    const bottom = node("div", undefined, "native-dock-bottom");
    bottom.append(progress, transport, finalActions);
    const options = node("div", undefined, "native-dock-options");
    const modes = node("div", undefined, "segmented");
    modes.append(button(t("text_mode"), () => send({ action: "dock_preview_mode", background: false })), button(t("background_mode"), () => send({ action: "dock_preview_mode", background: true })));
    modes.children[0].dataset.dockMode = "text";
    modes.children[0].dataset.i18n = "text_mode";
    modes.children[1].dataset.dockMode = "background";
    modes.children[1].dataset.i18n = "background_mode";
    const inspect = button("", () => {
        const dialog = $("native-output-dialog");
        const container = dialog?.querySelector(".native-output-cards");
        if (container && model.state?.live?.slides) {
            container.replaceChildren(...model.state.live.slides.map((text, index) => dockCard(text, index, model.state)));
        }
        dialog?.showModal();
    });
    inspect.append(icon("expand"), node("span", t("inspect_slides")));
    const inspectLabel = inspect.lastElementChild;
    if (inspectLabel)
        inspectLabel.dataset.i18n = "inspect_slides";
    const hide = button("", () => send({ action: "dock_visibility", visible: false }));
    hide.append(icon("eyeoff"), node("span", t("hide_panel")));
    const hideLabel = hide.lastElementChild;
    if (hideLabel)
        hideLabel.dataset.i18n = "hide_panel";
    options.append(modes, inspect, hide);
    row.append(options);
    const cards = node("div", undefined, "native-dock-cards");
    cards.id = "native-dock-cards";
    footer.append(row, cards, bottom);
    const dialog = node("dialog");
    dialog.id = "native-output-dialog";
    dialog.className = "native-output-dialog";
    const title = node("h2", t("inspect_slides"));
    title.dataset.i18n = "inspect_slides";
    dialog.append(title, node("div", undefined, "native-output-cards"), textButton("close", () => dialog.close()));
    document.body.append(dialog);
}
function prepareChrome() {
    const rail = document.querySelector(".app-rail");
    const home = rail.querySelector(".rail-home");
    const bible = rail.querySelector('[data-nav-kind="bible"]');
    const settings = $("layout-reset");
    settings.replaceChildren(icon("settings"));
    settings.dataset.tip = "settings";
    const appearance = $("appearance");
    const focus = $("focus-mode");
    const themes = $("themes-toggle");
    const settingsDialog = node("dialog");
    settingsDialog.id = "native-settings";
    const settingsTitle = node("h2", t("workspace"));
    settingsTitle.dataset.i18n = "workspace";
    const settingsActions = node("div", undefined, "native-settings-actions");
    settingsActions.append(textButton("appearance", () => appearance.click()), textButton("reset_layout", () => resetNativeLayout()), button("Sair do controle", () => $("control-logout").click()), textButton("close", () => settingsDialog.close()));
    settingsDialog.append(settingsTitle, settingsActions, appearance, focus);
    document.body.append(settingsDialog);
    appearance.classList.add("native-extra-tools");
    focus.classList.add("native-extra-tools");
    const bottom = node("div", undefined, "rail-bottom");
    bottom.append(settings, $("control-logout"));
    rail.replaceChildren(home, bible, themes, $("chat-link"), bottom);
    themes.onclick = () => {
        if (mobile()) {
            showView("themes");
        }
        else {
            const themesPanel = panelMap.Themes;
            const root = $("native-layout");
            if (themesPanel && !root.contains(themesPanel)) {
                resetNativeLayout();
            }
            else if (themesPanel) {
                themesPanel.scrollIntoView({ behavior: "smooth" });
            }
        }
    };
    home.onclick = () => {
        page = "home";
        setNativePage(page);
        document.querySelector('[data-kind="song"]')?.click();
        showView("operation", "preview");
    };
    bible.onclick = () => {
        page = "bible";
        setNativePage(page);
        document.querySelector('[data-kind="bible"]')?.click();
        showView("library");
    };
    const chatLink = $("chat-link");
    if (chatLink) {
        chatLink.onclick = (event) => {
            event.preventDefault();
            page = "chat";
            setNativePage(page);
            if (window.syncChatTheme)
                window.syncChatTheme();
        };
    }
    settings.onclick = () => settingsDialog.showModal();
    const headingH1 = document.querySelector(".app-heading h1");
    if (headingH1)
        headingH1.dataset.i18n = "native_shortcuts";
    const operator = $("operator-label");
    const operatorPill = node("span", undefined, "native-operator");
    operator.before(operatorPill);
    operatorPill.append(icon("user"), operator, icon("down"));
    const service = $("service");
    const servicePill = node("span", undefined, "native-service");
    service.before(servicePill);
    servicePill.append(icon("church"), service);
    const dateWrapper = $("service-date-wrapper");
    if (dateWrapper && $("service-date")) {
        $("service-date").before(icon("calendar"));
    }
    const screens = $("screens-open");
    screens.className = "native-screens";
    screens.replaceChildren(icon("screen"), node("span", "", "native-screen-count"));
    const quickEmpty = $("quick-empty");
    if (quickEmpty)
        quickEmpty.dataset.i18n = "native_no_shortcuts";
    const chrome = document.querySelector(".header-right");
    chrome.append(screens);
    const showDock = button("", () => send({ action: "dock_visibility", visible: true }));
    showDock.id = "native-show-dock";
    showDock.append(icon("screen"), node("span", t("presentation_controls")));
    const showDockLabel = showDock.lastElementChild;
    if (showDockLabel)
        showDockLabel.dataset.i18n = "presentation_controls";
    chrome.prepend(showDock);
}
export function translateNative() {
    document.querySelectorAll("[data-tip]").forEach((control) => {
        const tip = control.dataset.tip;
        if (tip) {
            control.title = t(tip);
            control.setAttribute("aria-label", t(tip));
        }
    });
    $("library-heading")?.setAttribute("data-i18n", "native_library");
    document.querySelectorAll("button[title]").forEach((control) => {
        const tip = control.dataset.tip || control.title;
        const translated = t(tip);
        if (translated && translated !== tip) {
            control.title = translated;
            control.setAttribute("aria-label", translated);
        }
    });
}
export function setupNative() {
    paintIcons();
    preparePanels();
    setupNativeBible(panelMap.Selection, panelMap.Lyrics);
    prepareChrome();
    document.body.dataset.nativeReady = "true";
    renderDocking(null, true);
    loadNativeIcons();
    document.querySelectorAll(".mobile-nav [data-view]").forEach((control) => (control.onclick = () => {
        if (control.dataset.mobileKind) {
            document
                .querySelector(`[data-kind="${control.dataset.mobileKind}"]`)
                ?.click();
            page = control.dataset.mobileKind === "bible" ? "bible" : "home";
            setNativePage(page);
        }
        showView(control.dataset.view || "operation", "live");
    }));
    document
        .querySelectorAll("[data-monitor]")
        .forEach((control) => (control.onclick = () => showView("operation", control.dataset.monitor)));
    const more = () => $("mobile-more")?.showModal();
    const mobileOptions = $("mobile-options");
    if (mobileOptions)
        mobileOptions.onclick = more;
    const mobileMoreOpen = $("mobile-more-open");
    if (mobileMoreOpen)
        mobileMoreOpen.onclick = more;
    document.querySelectorAll("#mobile-more [data-nav-kind]").forEach((control) => (control.onclick = () => {
        $("mobile-more")?.close();
        page = "home";
        setNativePage(page);
        document
            .querySelector(`[data-kind="${control.dataset.navKind}"]`)
            ?.click();
        showView("library");
    }));
    document.addEventListener("control-quick", (event) => {
        page = "home";
        setNativePage(page);
        const detail = event.detail;
        const itemKind = detail?.kind || "song";
        document.querySelector(`[data-kind="${itemKind}"]`)?.click();
    });
    const mobileThemes = $("mobile-themes");
    if (mobileThemes) {
        mobileThemes.onclick = () => {
            $("mobile-more")?.close();
            showView("themes");
            setTimeout(fitNativePreview, 50);
        };
    }
    $("themes-toggle").onclick = () => {
        if (mobile()) {
            showView("themes");
            setTimeout(fitNativePreview, 50);
        }
        else {
            const themesPanel = panelMap.Themes;
            const root = $("native-layout");
            if (themesPanel && !root.contains(themesPanel)) {
                resetNativeLayout();
            }
            else if (themesPanel) {
                themesPanel.scrollIntoView({ behavior: "smooth" });
            }
        }
    };
    const mobileScreens = $("mobile-screens");
    if (mobileScreens) {
        mobileScreens.onclick = () => {
            $("mobile-more")?.close();
            $("screens-dialog")?.showModal();
        };
    }
    $("mobile-chat")?.addEventListener("click", () => {
        $("mobile-more")?.close();
        page = "chat";
        setNativePage(page);
        if (window.syncChatTheme)
            window.syncChatTheme();
    });
    const mobileService = $("mobile-service");
    if (mobileService) {
        mobileService.onchange = () => send({ action: "service", id: mobileService.value }, mobileService);
    }
    $("mobile-service-date")?.addEventListener("change", () => {
        const val = $("mobile-service-date")?.value;
        send({ action: "service_date", date: val });
    });
    $("mobile-service-prev-date")?.addEventListener("click", () => send({ action: "service_step_date", delta: -1 }));
    $("mobile-service-next-date")?.addEventListener("click", () => send({ action: "service_step_date", delta: 1 }));
    const appearance = () => {
        const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
        model.themeOverride =
            next === (model.state?.dark ? "dark" : "light") ? null : next;
        if (model.themeOverride)
            localStorage.setItem("control-theme", next);
        else
            localStorage.removeItem("control-theme");
        if (model.state)
            updateNative(model.state);
        if (window.syncChatTheme)
            window.syncChatTheme();
    };
    $("appearance").onclick = appearance;
    const mobileApp = $("mobile-appearance");
    if (mobileApp)
        mobileApp.onclick = appearance;
    const viewport = () => {
        if (window.visualViewport?.scale === 1)
            document.documentElement.style.setProperty("--app-height", window.visualViewport.height + "px");
    };
    window.visualViewport?.addEventListener("resize", viewport);
    viewport();
    const resize = new ResizeObserver(fitNativePreview);
    if (panelMap.Preview)
        resize.observe(panelMap.Preview);
    if (panelMap.Stage)
        resize.observe(panelMap.Stage);
    if (panelMap.Themes)
        resize.observe(panelMap.Themes);
}
function fitNativePreview() {
    const themes = panelMap.Themes;
    const computed = themes ? getComputedStyle(themes) : null;
    const width = themes?.clientWidth
        ? themes.clientWidth -
            parseFloat(computed?.paddingLeft || "0") -
            parseFloat(computed?.paddingRight || "0")
        : mobile()
            ? window.innerWidth - 24
            : 0;
    if (themes && width > 0) {
        const columns = width >= 420 ? 4 : width >= 300 ? 3 : width >= 180 ? 2 : 1;
        themes.style.setProperty("--theme-columns", String(columns));
        themes.style.setProperty("--theme-height", Math.max(35, Math.round((width - (columns - 1) * 6) /
            columns /
            Number(model.state?.aspect || 16 / 9))) + "px");
    }
    for (const [panel, id] of [
        [panelMap.Preview, "preview-stage"],
        [panelMap.Stage, "live-stage"],
    ]) {
        if (!panel || !panel.clientWidth || !panel.clientHeight)
            continue;
        const stage = $(id);
        const panelComputed = getComputedStyle(panel);
        const panelWidth = panel.clientWidth -
            parseFloat(panelComputed.paddingLeft) -
            parseFloat(panelComputed.paddingRight);
        const footer = panel.querySelector(".monitor-footer");
        const headingEl = panel.querySelector(".panel-heading");
        const available = mobile()
            ? 240
            : Math.max(35, panel.clientHeight -
                parseFloat(panelComputed.paddingTop) -
                parseFloat(panelComputed.paddingBottom) -
                (headingEl?.offsetHeight || 0) -
                (footer?.hidden ? 0 : footer?.offsetHeight || 0) -
                10);
        const aspect = Number(model.state?.aspect || 16 / 9);
        const height = Math.min(panelWidth / aspect, available);
        stage.style.width = height * aspect + "px";
        stage.style.height = height + "px";
        stage.style.flex = "0 0 auto";
        stage.style.marginLeft = "auto";
        stage.style.marginRight = "auto";
    }
    const stage = $("preview-stage");
    const text = stage.firstElementChild;
    if (!stage.clientWidth || !stage.clientHeight || !text)
        return;
    let size = Math.max(12, Math.min(28, stage.clientWidth / 22));
    text.style.fontSize = size + "px";
    while (text.scrollHeight > stage.clientHeight * 0.8 && size > 10) {
        size -= 0.5;
        text.style.fontSize = size + "px";
    }
}
export function updateNative(state) {
    const previewIframe = $("native-preview-output");
    if (previewIframe && !previewIframe.getAttribute("src")) {
        previewIframe.src = "/screen/__control_preview";
    }
    const outputKey = JSON.stringify([
        state.preview?.index,
        state.preview?.title,
        state.preview?.theme?.id,
        state.live?.index,
        state.live?.title,
        state.live?.theme?.id,
        state.presenting,
        state.hidden,
        state.overlay,
    ]);
    if (outputKey !== outputSignature) {
        outputSignature = outputKey;
        for (const id of ["native-preview-output", "live-viewer"]) {
            const frame = $(id);
            frame?.contentWindow?.postMessage({ type: "control-refresh" }, location.origin);
        }
    }
    const selH2 = panelMap.Selection?.querySelector("h2");
    if (selH2) {
        selH2.textContent = t(page === "bible" ? "bible" : "native_library");
    }
    const lyrH2 = panelMap.Lyrics?.querySelector("h2");
    if (lyrH2) {
        lyrH2.textContent = t(page === "bible" ? "native_bible_text" : "native_lyrics");
    }
    document.documentElement.dataset.theme =
        model.themeOverride || (state.dark ? "dark" : "light");
    document.documentElement.style.setProperty("--native-scale", mobile() ? "1" : String(Math.max(0.1, (state.scale || 100) / 100)));
    document.body.dataset.nativeHydrated = "true";
    document.documentElement.style.setProperty("--aspect", String(state.aspect || 16 / 9));
    document.body.dataset.presenting = String(state.presenting && state.show_dock !== false);
    const showDockBtn = $("native-show-dock");
    if (showDockBtn) {
        showDockBtn.hidden = !state.presenting || state.show_dock !== false;
    }
    const progressFill = $("native-progress-fill");
    if (progressFill) {
        progressFill.style.width = (state.progress || 0) * 100 + "%";
    }
    const jumpInput = $("native-slide-jump");
    if (jumpInput && document.activeElement !== jumpInput) {
        jumpInput.value = state.live?.slides?.length
            ? String((state.live.index || 0) + 1)
            : "";
    }
    document
        .querySelectorAll("[data-dock-mode]")
        .forEach((control) => control.classList.toggle("active", state.dock_background === (control.dataset.dockMode === "background")));
    document.body.dataset.lyricsMode =
        (cardsOverride ?? state.lyrics_cards) ? "cards" : "list";
    document.body.dataset.previewDesign = String(!!state.preview?.design);
    const titleEl = document.querySelector(".native-song-title");
    if (titleEl)
        titleEl.textContent = state.preview?.title || "";
    const artistEl = document.querySelector(".native-song-artist");
    if (artistEl)
        artistEl.textContent = state.preview?.subtitle || "";
    const idleEl = $("native-live-idle");
    if (idleEl) {
        idleEl.textContent = t(state.hidden ? "native_hidden" : "native_ready");
        idleEl.hidden = !!(state.presenting && !state.hidden);
    }
    const footerEl = panelMap.Stage?.querySelector(".monitor-footer");
    if (footerEl) {
        footerEl.hidden = !state.presenting || !!state.hidden;
    }
    if (!state.presenting && state.idle_theme) {
        applyTheme($("live-stage"), state.idle_theme);
    }
    if (state.hidden) {
        const liveStage = $("live-stage");
        if (liveStage)
            liveStage.style.background = "#000";
    }
    $("player-panel").hidden = false;
    const playerEmpty = $("native-player-empty");
    if (playerEmpty) {
        playerEmpty.hidden = !!state.player;
        playerEmpty.querySelector("strong").textContent = t("empty_player");
        playerEmpty.querySelector(".native-player-hint").textContent =
            t("player_hint");
    }
    const playerControls = $("player-panel")?.querySelector(".player-controls");
    if (playerControls)
        playerControls.hidden = !state.player;
    const clearBtn = $("playlist-clear");
    if (clearBtn) {
        clearBtn.disabled = !state.playlist?.some((item) => item.selected);
    }
    const screenCount = document.querySelector(".native-screen-count");
    if (screenCount) {
        screenCount.textContent =
            (state.screens?.length || 0) + " " + t("native_web_screens");
    }
    const transportTitle = $("transport-title");
    if (transportTitle) {
        transportTitle.textContent =
            state.live?.design || state.live?.title || t("no_content");
    }
    const transportPos = $("transport-position");
    if (transportPos) {
        const slidesLen = state.live?.slides?.length || 0;
        const curIdx = (state.live?.index || 0) + 1;
        transportPos.textContent = !state.presenting
            ? t("stopped")
            : mobile()
                ? `${curIdx} / ${slidesLen}`
                : `${t("strophe")} ${curIdx} ${t("of")} ${slidesLen}`;
    }
    const transportDot = $("transport-dot");
    if (transportDot) {
        transportDot.style.opacity =
            state.presenting && !state.hidden ? "1" : ".25";
    }
    let hideLabel = $("hide-toggle")?.querySelector(".native-button-label");
    if (!hideLabel) {
        hideLabel = node("span", "", "native-button-label");
        $("hide-toggle")?.append(hideLabel);
    }
    hideLabel.textContent = t(state.hidden ? "show_screen" : "hide_screen");
    const select = $("mobile-service");
    if (select) {
        if (select.dataset.items !== JSON.stringify(state.services)) {
            select.replaceChildren(...Array.from($("service")?.options || [], (opt) => opt.cloneNode(true)));
            select.dataset.items = JSON.stringify(state.services);
        }
        if (document.activeElement !== select && state.service) {
            select.value = state.service;
        }
    }
    const dateSelect = $("mobile-service-date");
    if (dateSelect && $("service-date")) {
        if (dateSelect.dataset.items !== JSON.stringify(state.service_dates)) {
            dateSelect.replaceChildren(...Array.from($("service-date")?.options || [], (opt) => opt.cloneNode(true)));
            dateSelect.dataset.items = JSON.stringify(state.service_dates);
        }
        if (document.activeElement !== dateSelect && state.service_date) {
            dateSelect.value = state.service_date;
        }
    }
    document
        .querySelector(".rail-home")
        ?.classList.toggle("active", page === "home");
    document
        .querySelector('.app-rail [data-nav-kind="bible"]')
        ?.classList.toggle("active", page === "bible");
    document
        .querySelector(".app-rail #chat-link")
        ?.classList.toggle("active", page === "chat");
    if (dockSignature !==
        JSON.stringify([
            state.live?.slides,
            state.live?.index,
            state.dock_background,
            state.live?.theme?.id,
        ])) {
        dockSignature = JSON.stringify([
            state.live?.slides,
            state.live?.index,
            state.dock_background,
            state.live?.theme?.id,
        ]);
        const dockCardsEl = $("native-dock-cards");
        if (dockCardsEl && state.live?.slides) {
            dockCardsEl.replaceChildren(...state.live.slides.map((text, index) => dockCard(text, index, state)));
        }
        const dialog = $("native-output-dialog");
        if (dialog?.open && state.live?.slides) {
            dialog
                .querySelector(".native-output-cards")
                ?.replaceChildren(...state.live.slides.map((text, index) => dockCard(text, index, state)));
        }
    }
    renderDocking(state);
    fitNativePreview();
}
function dockCard(text, index, state) {
    const card = button("", () => send({ action: "live_slide", index }), "native-dock-card" + (index === state.live?.index ? " active" : ""));
    const face = node("span", undefined, "native-dock-face");
    face.append(node("span", text));
    if (state.dock_background)
        applyTheme(face, state.live?.theme);
    card.append(face, node("small", String(index + 1)));
    return card;
}
export function openNativeMenu(anchor, actions) {
    document.querySelector(".native-context-menu")?.remove();
    const menu = node("div", undefined, "native-context-menu");
    menu.setAttribute("role", "menu");
    const rect = anchor instanceof Element
        ? anchor.getBoundingClientRect()
        : { left: anchor.x, bottom: anchor.y };
    for (const item of actions) {
        const row = button("", () => {
            menu.remove();
            item.action();
        });
        row.setAttribute("role", "menuitem");
        row.disabled = !!item.disabled;
        row.append(icon(item.symbol), node("span", item.label));
        menu.append(row);
    }
    document.body.append(menu);
    const scale = mobile() ? 1 : ((model.state?.scale || 100) / 100);
    menu.style.left =
        Math.max(4, Math.min(innerWidth - menu.getBoundingClientRect().width - 4, rect.left)) /
            scale +
            "px";
    menu.style.top =
        Math.max(4, Math.min(innerHeight - menu.getBoundingClientRect().height - 4, rect.bottom)) /
            scale +
            "px";
    setTimeout(() => document.addEventListener("pointerdown", (event) => {
        if (!menu.contains(event.target))
            menu.remove();
    }, { once: true }), 0);
}
