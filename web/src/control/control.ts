import {
  $,
  model,
  t,
  node,
  button,
  iconButton,
  setControlLabel,
  revealInList,
  request,
  send,
  accept,
  applyTheme,
  showView,
  toast,
} from "/control-api.js";
import {
  setupLibrary,
  refreshLibrary,
  highlightLibrary,
} from "/control-library.js";
import { setupBible, openBible } from "/control-bible.js";
import {
  setupInterface,
  translateChrome,
  updateChrome,
  openNativeMenu,
} from "/control-ui.js";
import { icon } from "/control-icons.js";
import type { ControlState, PlaylistItem } from "../types/state.js";

declare global {
  interface Window {
    setNativePage?: (page: string) => void;
  }
}

let source: "preview" | "live" = "preview";
let language: string | undefined;
let catalogRevision: string | undefined;
let polling = false;
const stamps = new Map<string, string>();
let slideStructure = "";
let playlistStructure = "";

function changed(key: string, value: unknown, render: () => void): void {
  const stamp = JSON.stringify(value);
  if (stamps.get(key) !== stamp) {
    stamps.set(key, stamp);
    render();
  }
}

function translated(): void {
  document.querySelectorAll<HTMLElement>("[data-i18n]").forEach((element) => {
    const key = element.dataset.i18n;
    if (key) element.textContent = t(key);
  });
  document
    .querySelectorAll<HTMLInputElement>("[data-placeholder]")
    .forEach((element) => {
      const ph = element.dataset.placeholder;
      if (ph) element.placeholder = t(ph);
    });
  document.title = t("title");
  translateChrome();
}

function connection(online: boolean): void {
  const connEl = $("connection");
  if (connEl) {
    connEl.textContent = t(online ? "connected" : "offline");
    connEl.className = "connection " + (online ? "online" : "offline");
  }
}

function renderState(state: ControlState): void {
  if (!state.ready) return;
  connection(true);
  const operatorEl = $("operator-label");
  if (operatorEl && state.operator) operatorEl.textContent = state.operator;
  const chatLink = $("chat-link");
  if (chatLink) chatLink.hidden = !state.chat_enabled;
  const syncTime = $("sync-time");
  if (syncTime) {
    syncTime.textContent =
      t("synced") + " " + new Date().toLocaleTimeString();
  }

  changed("services", state.services, () => {
    if (state.services) {
      $<HTMLSelectElement>("service")?.replaceChildren(
        ...state.services.map((service) => {
          const option = node("option", service.name);
          option.value = service.id;
          return option;
        }),
      );
    }
  });

  const serviceSelect = $<HTMLSelectElement>("service");
  if (serviceSelect && document.activeElement !== serviceSelect && state.service) {
    serviceSelect.value = state.service;
  }

  changed("service_dates", [state.service_dates, state.service], () => {
    const serviceDateSelect = $<HTMLSelectElement>("service-date");
    if (state.service_dates && serviceDateSelect) {
      serviceDateSelect.replaceChildren(
        ...state.service_dates.map((d) => {
          const count = d.items > 0 ? ` (${d.items})` : "";
          const option = node("option", `${d.label}${count}`);
          option.value = d.date;
          return option;
        }),
      );
    }
  });

  const sDateSelect = $<HTMLSelectElement>("service-date");
  if (
    sDateSelect &&
    document.activeElement !== sDateSelect &&
    state.service_date
  ) {
    sDateSelect.value = state.service_date;
  }

  for (const key of ["preview", "live"] as const) {
    const item = state[key];
    if (item) {
      changed(key, [item, state.hidden, state.presenting], () => {
        const stage = $(key + "-stage");
        applyTheme(stage, item.theme);
        const first = stage.firstElementChild;
        if (first) {
          first.textContent =
            item.design ||
            (item.slides && item.index !== undefined ? item.slides[item.index] : undefined) ||
            (item.title ? "" : t("no_content"));
        }
        stage.classList.toggle(
          "hidden",
          key === "live" && (!state.presenting || !!state.hidden),
        );
        const titleEl = $(key + "-title");
        if (titleEl) {
          titleEl.textContent = item.design || item.title || t("no_content");
        }
        const subtitleEl = $(key + "-subtitle");
        if (subtitleEl) subtitleEl.textContent = item.subtitle || "";
        const countEl = $(key + "-count");
        if (countEl) {
          countEl.textContent = item.slides?.length
            ? `${(item.index || 0) + 1} / ${item.slides.length}`
            : "";
        }
      });
    }
  }

  const liveStatus = $("live-status");
  if (liveStatus) {
    liveStatus.textContent = t(
      !state.presenting ? "stopped" : state.hidden ? "hidden" : "live",
    );
    liveStatus.className = "badge " + (state.presenting ? "live" : "");
  }
  const liveViewer = $<HTMLIFrameElement>("live-viewer");
  if (liveViewer) {
    if (!liveViewer.getAttribute("src")) {
      liveViewer.src = "/screen/__control_live";
    }
    liveViewer.hidden = !!state.hidden;
  }

  setControlLabel(
    $("hide-toggle"),
    t(state.hidden ? "show" : "hide"),
    state.hidden ? "eye" : "eyeoff",
  );
  $("hide-toggle")?.classList.toggle("active", !!state.hidden);
  const editSongBtn = $("edit-song");
  if (editSongBtn) editSongBtn.hidden = !state.preview?.song_id;
  const projectPreviewBtn = $<HTMLButtonElement>("project-preview");
  if (projectPreviewBtn) {
    projectPreviewBtn.disabled =
      !state.preview?.slides?.length && !state.preview?.design;
  }

  document.querySelectorAll<HTMLButtonElement>("[data-direction]").forEach((control) => {
    const slidesLen = state.live?.slides?.length || 0;
    const curIdx = state.live?.index || 0;
    control.disabled =
      !state.presenting ||
      !slidesLen ||
      (["first", "previous"].includes(control.dataset.direction || "")
        ? curIdx === 0
        : curIdx + 1 >= slidesLen);
  });

  renderSlides();
  changed("playlist", state.playlist, renderPlaylist);

  changed("quick", state.quick, () => {
    const quickItems = state.quick || [];
    const quickEmpty = $("quick-empty");
    if (quickEmpty) quickEmpty.hidden = !!quickItems.length;
    const mobileSec = $("mobile-quick-items")?.closest("section");
    if (mobileSec) mobileSec.hidden = !quickItems.length;

    for (const id of ["quick-items", "mobile-quick-items"]) {
      $(id)?.replaceChildren(
        ...quickItems.map((item) => {
          const group = node("span", undefined, "quick-pill");
          const activate = button(item.title, async () => {
            if (await send({ action: "quick_select", id: item.id })) {
              document.dispatchEvent(
                new CustomEvent("control-quick", {
                  detail: {
                    kind:
                      ({
                        Song: "song",
                        Text: "text",
                        Verse: "verse",
                        Media: "media",
                        Design: "design",
                      } as Record<string, string>)[item.kind || ""] || "song",
                  },
                }),
              );
              $<HTMLDialogElement>("mobile-more")?.close();
              showView("operation", "preview");
            }
          });
          activate.ondblclick = () =>
            send({ action: "quick_select", id: item.id, project: true });
          const remove = iconButton(t("remove"), "close", (control) =>
            send({ action: "quick_remove", id: item.id }, control),
          );
          remove.setAttribute("aria-label", t("remove") + " " + item.title);
          group.append(activate, remove);
          return group;
        }),
      );
    }
  });

  changed("themes", [state.themes, state.preview?.theme?.id], () => {
    const themesEl = $("themes");
    if (!themesEl || !state.themes) return;
    themesEl.replaceChildren(
      ...state.themes.map((theme) => {
        const liveCheck = $<HTMLInputElement>("theme-live");
        const control = button(
          "",
          (element) =>
            send(
              { action: "theme", id: theme.id, live: liveCheck?.checked ?? false },
              element,
            ),
          "theme-card" + (theme.id === state.preview?.theme?.id ? " active" : ""),
        );
        const swatch = node("span", "Aa", "theme-swatch");
        applyTheme(swatch, theme);
        control.append(swatch, node("span", theme.name));
        control.dataset.id = theme.id;
        control.title = theme.name;
        control.ondblclick = () =>
          send({ action: "theme", id: theme.id, live: true });
        control.oncontextmenu = (event: MouseEvent) => {
          event.preventDefault();
          openNativeMenu({ x: event.clientX, y: event.clientY }, [
            {
              label: t("preview"),
              symbol: "eye",
              action: () => send({ action: "theme", id: theme.id }),
            },
            {
              label: t("apply_live"),
              symbol: "play",
              action: () => send({ action: "theme", id: theme.id, live: true }),
            },
          ]);
        };
        return control;
      }),
    );
  });

  const playerPanel = $("player-panel");
  if (playerPanel) playerPanel.hidden = !state.player;
  if (state.player) {
    const pTitle = $("player-title");
    if (pTitle) pTitle.textContent = state.player.title;
    setControlLabel(
      $("player-toggle"),
      t(state.player.playing ? "pause_timer" : "play"),
      state.player.playing ? "pause" : "play",
    );
    const pSeek = $<HTMLInputElement>("player-seek");
    if (pSeek && document.activeElement !== pSeek) {
      pSeek.max = String(state.player.duration || 1);
      pSeek.value = String(state.player.position);
    }
    const pVol = $<HTMLInputElement>("player-volume");
    if (pVol && document.activeElement !== pVol) {
      pVol.value = String(state.player.volume);
    }
    const pTime = $("player-time");
    if (pTime) {
      pTime.textContent =
        clock(state.player.position) + " / " + clock(state.player.duration);
    }
  }

  const timers = state.timers || [];
  const designControls = $("design-controls");
  if (designControls) {
    designControls.hidden =
      !timers.length && !state.overlay && !state.questions;
  }
  const overlayName = $("overlay-name");
  if (overlayName) overlayName.textContent = state.overlay || "";
  const overlayClear = $("overlay-clear");
  if (overlayClear) overlayClear.hidden = !state.overlay;
  const questionControls = $("question-controls");
  if (questionControls) questionControls.hidden = !state.questions;

  document
    .querySelectorAll<HTMLElement>(
      '[data-design-op="start"],[data-design-op="pause"],[data-design-op="reset"]',
    )
    .forEach((control) => (control.hidden = !timers.length));

  changed("timers", state.timers, () => {
    $("timers")?.replaceChildren(
      ...timers.map((timer) => {
        const row = node("div", undefined, "timer");
        row.dataset.running = String(timer.running);
        row.append(
          node("span", timer.name),
          node("small", t(timer.running ? "running" : "paused")),
          node("strong", timer.text),
        );
        return row;
      }),
    );
  });

  const startOp = document.querySelector<HTMLButtonElement>('[data-design-op="start"]');
  if (startOp) startOp.disabled = timers.every((timer) => timer.running);
  const pauseOp = document.querySelector<HTMLButtonElement>('[data-design-op="pause"]');
  if (pauseOp) pauseOp.disabled = timers.every((timer) => !timer.running);

  changed("screens", state.screens, () => {
    const screens = state.screens || [];
    const screensEl = $("screens");
    if (!screensEl) return;
    screensEl.replaceChildren(
      ...screens.map((screen) => {
        const link = node("a", screen.name + " ↗", "button");
        link.href = "/screen/" + encodeURIComponent(screen.slug || screen.id);
        link.target = "_blank";
        link.rel = "noopener";
        return link;
      }),
    );
    if (!screens.length) {
      screensEl.append(node("p", t("no_screens"), "empty"));
    }
  });

  updateChrome(state);
  highlightLibrary();
  if (catalogRevision !== state.catalog_revision) {
    catalogRevision = state.catalog_revision;
    refreshLibrary();
  }
}

function renderSlides(): void {
  const state = model.state;
  const item = state?.[source];
  if (!item || !state) return;

  changed(
    "slides",
    [
      source,
      (model as unknown as { cardsOverride?: boolean }).cardsOverride ?? state.lyrics_cards,
      item.slides,
      item.index,
      state.live?.title,
      state.live?.subtitle,
      state.live?.index,
    ],
    () => {
      const slides = item.slides || [];
      const structure = JSON.stringify([source, slides]);
      if (
        structure === slideStructure &&
        $("slides")?.querySelectorAll(".slide-card").length === slides.length
      ) {
        $("slides")
          ?.querySelectorAll<HTMLElement>(".slide-card")
          .forEach((card, index) => {
            card.classList.toggle("active", index === item.index);
            card.classList.toggle(
              "on-air",
              index === state.live?.index &&
                item.title === state.live?.title &&
                item.subtitle === state.live?.subtitle,
            );
          });
        revealInList(
          $("slides"),
          $("slides")?.querySelector<HTMLElement>(".slide-card.active"),
        );
        return;
      }
      slideStructure = structure;
      const slidesEl = $("slides");
      if (!slidesEl) return;
      slidesEl.replaceChildren(
        ...slides.map((text, index) => {
          const card = node(
            "article",
            undefined,
            "slide-card" + (index === item.index ? " active" : ""),
          );
          if (
            index === state.live?.index &&
            item.title === state.live?.title &&
            item.subtitle === state.live?.subtitle
          ) {
            card.classList.add("on-air");
          }
          const select = button(
            "",
            () =>
              send({
                action: source === "preview" ? "preview_slide" : "live_slide",
                index,
              }),
            "slide-select",
          );
          select.append(
            node("small", String(index + 1)),
            node("span", text, "slide-text"),
          );
          select.ondblclick = () =>
            send({
              action: source === "preview" ? "project_preview" : "live_slide",
              index,
            });
          const project = iconButton(
            t(source === "preview" ? "project" : "select_live"),
            "play",
            (control) =>
              send(
                {
                  action:
                    source === "preview" ? "project_preview" : "live_slide",
                  index,
                },
                control,
              ),
            "project-slide",
          );
          card.append(select, project);
          return card;
        }),
      );
      if (!slides.length) {
        slidesEl.append(
          node("p", item.design || t("select_content"), "empty"),
        );
      }
    },
  );
}

function renderPlaylist(): void {
  const items: PlaylistItem[] = model.state?.playlist || [];
  const countEl = $("playlist-count");
  if (countEl) countEl.textContent = String(items.length);
  const structure = JSON.stringify(
    items.map((item) => [item.id, item.title, item.subtitle, item.kind]),
  );
  const playlistEl = $("playlist");
  if (!playlistEl) return;

  if (
    structure === playlistStructure &&
    playlistEl.querySelectorAll(".playlist-row").length === items.length
  ) {
    playlistEl.querySelectorAll<HTMLElement>(".playlist-row").forEach((row, index) => {
      row.classList.toggle("selected", !!items[index].selected);
      row.classList.toggle("played", !!items[index].played);
      const statusEl = row.querySelector<HTMLElement>(".playlist-status");
      if (statusEl) {
        setControlLabel(
          statusEl,
          t("mark_played"),
          items[index].played ? "check_circle" : "circle",
        );
      }
    });
    const selected = items.findIndex((item) => item.selected);
    const nextBtn = $<HTMLButtonElement>("playlist-next");
    if (nextBtn) nextBtn.disabled = !items.length || selected === items.length - 1;
    const prevBtn = $<HTMLButtonElement>("playlist-previous");
    if (prevBtn) prevBtn.disabled = selected <= 0;
    return;
  }

  playlistStructure = structure;
  playlistEl.replaceChildren(
    ...items.map((item, index) => {
      const row = node(
        "article",
        undefined,
        "playlist-row" +
          (item.selected ? " selected" : "") +
          (item.played ? " played" : ""),
      );
      row.dataset.id = String(item.id);
      row.draggable = true;
      row.ondragstart = (event: DragEvent) => {
        event.dataTransfer?.setData("application/x-control-playlist", String(item.id));
        if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
      };
      row.ondragover = (event: DragEvent) => {
        if (
          event.dataTransfer?.types.includes("application/x-control-playlist")
        ) {
          event.preventDefault();
          row.classList.add("drag-over");
        }
      };
      row.ondragleave = () => row.classList.remove("drag-over");
      row.ondrop = (event: DragEvent) => {
        event.preventDefault();
        row.classList.remove("drag-over");
        const id = event.dataTransfer?.getData("application/x-control-playlist");
        if (id && id !== String(item.id)) {
          send({
            action: "playlist_move",
            id,
            target: item.id,
            after:
              event.clientY >
              row.getBoundingClientRect().top + row.offsetHeight / 2,
          });
        }
      };
      const select = button(
        "",
        () => send({ action: "playlist_select", id: item.id }),
        "resource-title",
      );
      select.ondblclick = () =>
        send({ action: "playlist_select", id: item.id, project: true });
      const label = node("span");
      label.append(node("strong", item.title), node("small", item.subtitle));
      select.append(
        (() => {
          const mark = node("span", undefined, "position");
          mark.append(
            icon(
              ({
                Song: "music",
                Bible: "bible",
                Media: "media",
                Text: "text",
                Design: "design",
              } as Record<string, string>)[item.kind || ""] || "music",
            ),
          );
          return mark;
        })(),
        label,
      );
      const actions = node("div", undefined, "row-actions");
      const up = iconButton(t("move_up"), "up", (control) =>
        send(
          { action: "playlist_move", id: item.id, target: items[index - 1].id },
          control,
        ),
      );
      up.disabled = index === 0;
      up.setAttribute("aria-label", t("move_up"));
      const down = iconButton(t("move_down"), "down", (control) =>
        send(
          {
            action: "playlist_move",
            id: item.id,
            target: items[index + 1].id,
            after: true,
          },
          control,
        ),
      );
      down.disabled = index + 1 === items.length;
      down.setAttribute("aria-label", t("move_down"));
      const played = iconButton(
        t("mark_played"),
        item.played ? "check_circle" : "circle",
        (control) =>
          send(
            {
              action: "playlist_played",
              id: item.id,
              played: !model.state?.playlist?.find(
                (entry) => entry.id === item.id,
              )?.played,
            },
            control,
          ),
      );
      played.title = t("mark_played");
      played.classList.add("playlist-status");
      const remove = iconButton(t("remove"), "trash", (control) =>
        send({ action: "playlist_remove", id: item.id }, control),
      );
      remove.setAttribute("aria-label", t("remove") + " " + item.title);
      actions.append(
        up,
        down,
        remove,
        iconButton(
          t("project"),
          "play",
          (control) =>
            send(
              { action: "playlist_select", id: item.id, project: true },
              control,
            ),
          "primary",
        ),
      );
      const menu = (anchor: HTMLElement | { x: number; y: number }) =>
        openNativeMenu(anchor, [
          {
            label: t("project"),
            symbol: "play",
            action: () =>
              send({ action: "playlist_select", id: item.id, project: true }),
          },
          {
            label: t("move_up"),
            symbol: "up",
            disabled: index === 0,
            action: () =>
              send({
                action: "playlist_move",
                id: item.id,
                target: items[index - 1].id,
              }),
          },
          {
            label: t("move_down"),
            symbol: "down",
            disabled: index + 1 === items.length,
            action: () =>
              send({
                action: "playlist_move",
                id: item.id,
                target: items[index + 1].id,
                after: true,
              }),
          },
          {
            label: t("mark_played"),
            symbol: "check",
            action: () =>
              send({
                action: "playlist_played",
                id: item.id,
                played: !model.state?.playlist?.find(
                  (entry) => entry.id === item.id,
                )?.played,
              }),
          },
          {
            label: t("remove"),
            symbol: "trash",
            action: () => send({ action: "playlist_remove", id: item.id }),
          },
        ]);
      row.oncontextmenu = (event: MouseEvent) => {
        event.preventDefault();
        menu({ x: event.clientX, y: event.clientY });
      };
      const more = iconButton(t("more"), "more", (control) => menu(control), "native-row-more");
      row.append(select, played, more, actions);
      return row;
    }),
  );
  if (!items.length) {
    playlistEl.append(node("p", t("empty_playlist"), "empty"));
  }
  const selected = items.findIndex((item) => item.selected);
  const nextBtn = $<HTMLButtonElement>("playlist-next");
  if (nextBtn) nextBtn.disabled = !items.length || selected === items.length - 1;
  const prevBtn = $<HTMLButtonElement>("playlist-previous");
  if (prevBtn) prevBtn.disabled = selected <= 0;
}

function clock(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds || 0));
  return (
    Math.floor(s / 60) + ":" + String(s % 60).padStart(2, "0")
  );
}

setupBible();
setupLibrary(openBible);
model.onState = renderState;
const errEl = $("error");
if (errEl) errEl.onclick = () => (errEl.hidden = true);

const projPrev = $<HTMLButtonElement>("project-preview");
if (projPrev) {
  projPrev.onclick = async () => {
    if (await send({ action: "project_preview" }, projPrev)) {
      showView("operation", "live");
    }
  };
}

function haptic(ms = 12): void {
  if (typeof navigator !== "undefined" && navigator.vibrate) {
    try {
      navigator.vibrate(ms);
    } catch {
      /* ignore */
    }
  }
}

const hideToggle = $<HTMLButtonElement>("hide-toggle");
if (hideToggle) {
  hideToggle.onclick = () => {
    haptic(16);
    send(
      { action: "visibility", hidden: !model.state?.hidden },
      hideToggle,
    );
  };
}

const stopBtn = $<HTMLButtonElement>("stop");
if (stopBtn) {
  stopBtn.onclick = () => {
    haptic(20);
    send({ action: "stop" }, stopBtn);
  };
}

const serviceEl = $<HTMLSelectElement>("service");
if (serviceEl) {
  serviceEl.onchange = () =>
    send(
      { action: "service", id: serviceEl.value },
      serviceEl as unknown as HTMLButtonElement,
    );
}

const sDateEl = $<HTMLSelectElement>("service-date");
if (sDateEl) {
  sDateEl.onchange = () =>
    send(
      { action: "service_date", date: sDateEl.value },
      sDateEl as unknown as HTMLButtonElement,
    );
}

const sPrevDate = $<HTMLButtonElement>("service-prev-date");
if (sPrevDate) {
  sPrevDate.onclick = () =>
    send({ action: "service_step_date", delta: -1 }, sPrevDate);
}

const sNextDate = $<HTMLButtonElement>("service-next-date");
if (sNextDate) {
  sNextDate.onclick = () =>
    send({ action: "service_step_date", delta: 1 }, sNextDate);
}

const overlayClr = $<HTMLButtonElement>("overlay-clear");
if (overlayClr) {
  overlayClr.onclick = () =>
    send({ action: "overlay_clear" }, overlayClr);
}

const playlistClr = $<HTMLButtonElement>("playlist-clear");
if (playlistClr) {
  playlistClr.onclick = () =>
    send({ action: "playlist_clear" }, playlistClr);
}

for (const [id, offset] of [
  ["playlist-next", 1],
  ["playlist-previous", -1],
] as const) {
  const btn = $<HTMLButtonElement>(id);
  if (btn) {
    btn.onclick = () => {
      const items = model.state?.playlist || [];
      const selIdx = items.findIndex((item) => item.selected);
      const target = items[Math.max(0, selIdx + offset)];
      if (target) {
        haptic(10);
        send({ action: "playlist_select", id: target.id, project: true }, btn);
      }
    };
  }
}

document.querySelectorAll<HTMLButtonElement>("[data-direction]").forEach((control) => {
  control.onclick = () => {
    haptic(10);
    send(
      { action: "navigate", direction: control.dataset.direction },
      control,
    );
  };
});

document.querySelectorAll<HTMLElement>("[data-slide-source]").forEach((control) => {
  control.onclick = () => {
    source = (control.dataset.slideSource as "preview" | "live") || "preview";
    document
      .querySelectorAll("[data-slide-source]")
      .forEach((item) => item.classList.toggle("active", item === control));
    renderSlides();
  };
});

document.querySelectorAll<HTMLButtonElement>("[data-design-op]").forEach((control) => {
  control.onclick = () =>
    send(
      { action: "timer", operation: control.dataset.designOp },
      control,
    );
});

document.querySelectorAll<HTMLElement>(".mobile-nav button").forEach((control) => {
  control.onclick = () => showView(control.dataset.view || "operation");
});

document.querySelectorAll<HTMLElement>("[data-close]").forEach((control) => {
  control.onclick = () => control.closest<HTMLDialogElement>("dialog")?.close();
});

const noticeOpen = $("notice-open");
if (noticeOpen) {
  noticeOpen.onclick = () => {
    const diag = $<HTMLDialogElement>("text-dialog");
    diag?.querySelector(".dialog-error")?.remove();
    diag?.showModal();
    $<HTMLInputElement>("text-title")?.focus();
  };
}

const textForm = $<HTMLFormElement>("text-form");
if (textForm) {
  textForm.onsubmit = async (event: SubmitEvent) => {
    event.preventDefault();
    const submitBtn = event.submitter as HTMLButtonElement | null;
    const intent = submitBtn?.value;
    const ok = await send(
      {
        action: "text",
        title: $<HTMLInputElement>("text-title")?.value || "",
        content: $<HTMLTextAreaElement>("text-content")?.value || "",
        project: intent === "project",
        playlist: intent === "playlist",
      },
      submitBtn,
    );
    if (ok) {
      $<HTMLDialogElement>("text-dialog")?.close();
      if (intent === "playlist") toast(t("added"));
      else {
        document.dispatchEvent(
          new CustomEvent("control-quick", { detail: { kind: "text" } }),
        );
        showView("operation", intent === "project" ? "live" : "preview");
      }
    }
  };
}

function syncChatTheme(): void {
  const frame =
    $<HTMLIFrameElement>("native-chat-frame") ||
    $<HTMLIFrameElement>("chat-frame");
  try {
    if (frame?.contentDocument?.documentElement) {
      const theme = document.documentElement.dataset.theme;
      if (theme) frame.contentDocument.documentElement.dataset.theme = theme;
    }
  } catch {
    /* ignore */
  }
}

window.syncChatTheme = syncChatTheme;

const screensOpen = $("screens-open");
if (screensOpen) {
  screensOpen.onclick = () =>
    $<HTMLDialogElement>("screens-dialog")?.showModal();
}

const chatLink = $("chat-link");
if (chatLink) {
  chatLink.onclick = (event: MouseEvent) => {
    event.preventDefault();
    if (window.setNativePage) {
      window.setNativePage("chat");
      syncChatTheme();
    } else {
      location.href = "/chat";
    }
  };
}

const pToggle = $<HTMLButtonElement>("player-toggle");
if (pToggle) {
  pToggle.onclick = () =>
    send({ action: "player", operation: "toggle" }, pToggle);
}

const pStop = $<HTMLButtonElement>("player-stop");
if (pStop) {
  pStop.onclick = () =>
    send({ action: "player", operation: "stop" }, pStop);
}

const pSeek = $<HTMLInputElement>("player-seek");
if (pSeek) {
  pSeek.onchange = () =>
    send({
      action: "player",
      operation: "seek",
      value: Number(pSeek.value),
    });
}

const pVol = $<HTMLInputElement>("player-volume");
if (pVol) {
  pVol.onchange = () =>
    send({
      action: "player",
      operation: "volume",
      value: Number(pVol.value),
    });
}

document.addEventListener("keydown", (event: KeyboardEvent) => {
  if (
    event.isComposing ||
    (event.target as HTMLElement | null)?.isContentEditable ||
    /INPUT|TEXTAREA|SELECT/.test((event.target as HTMLElement)?.tagName || "") ||
    document.querySelector("dialog[open]")
  )
    return;
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "f") {
    event.preventDefault();
    showView("library");
    document.body.dataset.searchOpen = "true";
    $<HTMLInputElement>("library-search")?.focus();
    return;
  }
  if (event.ctrlKey || event.metaKey || event.altKey || !model.online) return;
  if (event.key === "Escape" && model.state?.presenting) {
    event.preventDefault();
    send({ action: "stop" });
    return;
  }
  if (
    event.code === "Space" &&
    (event.target as HTMLElement | null)?.closest("button,a")
  )
    return;

  const direction = ({
    ArrowRight: "next",
    ArrowDown: "next",
    " ": "next",
    PageDown: "next",
    ArrowLeft: "previous",
    ArrowUp: "previous",
    PageUp: "previous",
    Home: "first",
    End: "last",
  } as Record<string, string>)[event.key];

  if (direction && model.state?.presenting) {
    event.preventDefault();
    send({ action: "navigate", direction });
  }
  if (event.key.toLowerCase() === "h") {
    event.preventDefault();
    send({ action: "visibility", hidden: !model.state?.hidden });
  }
});

function setupMobileGestures(): void {
  let startX = 0;
  let startY = 0;
  let startTime = 0;

  window.addEventListener(
    "touchstart",
    (event: TouchEvent) => {
      if (event.touches.length !== 1) return;
      const touch = event.touches[0];
      const target = touch.target as HTMLElement | null;
      if (
        target?.closest(
          "input, select, textarea, button, dialog, .native-context-menu, #quick-items, .segmented, .bible-choices",
        )
      ) {
        startTime = 0;
        return;
      }
      startX = touch.clientX;
      startY = touch.clientY;
      startTime = Date.now();
    },
    { passive: true },
  );

  window.addEventListener(
    "touchend",
    (event: TouchEvent) => {
      if (!startTime || event.changedTouches.length !== 1) return;
      const touch = event.changedTouches[0];
      const deltaX = touch.clientX - startX;
      const deltaY = touch.clientY - startY;
      const elapsed = Date.now() - startTime;
      startTime = 0;

      if (
        elapsed < 500 &&
        Math.abs(deltaX) > 40 &&
        Math.abs(deltaX) > Math.abs(deltaY) * 1.3
      ) {
        if (model.state?.presenting && model.state?.live?.slides?.length) {
          if (deltaX < 0) {
            haptic(12);
            send({ action: "navigate", direction: "next" });
          } else {
            haptic(12);
            send({ action: "navigate", direction: "previous" });
          }
        }
      }
    },
    { passive: true },
  );
}

setupMobileGestures();

async function poll(): Promise<void> {
  if (polling) return;
  polling = true;
  try {
    const state = await request<ControlState>("state");
    if (state.ready && language !== state.language && state.language) {
      model.texts = await request<Record<string, string>>(
        "translations?lang=" + encodeURIComponent(state.language),
      );
      language = state.language;
      catalogRevision = undefined;
      document.documentElement.lang = language;
      translated();
      stamps.clear();
    }
    accept(state);
  } catch {
    model.online = false;
    connection(false);
  } finally {
    polling = false;
    setTimeout(poll, document.hidden ? 1500 : 300);
  }
}

document.addEventListener("visibilitychange", () => {
  if (!document.hidden) poll();
});

setupInterface();

document.addEventListener("control-slides-layout", () => {
  stamps.delete("slides");
  renderSlides();
  if (model.state) updateChrome(model.state);
});

poll();

