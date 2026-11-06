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
  report,
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

let source = "preview",
  language,
  catalogRevision,
  polling = false;
const stamps = new Map();
let slideStructure = "",
  playlistStructure = "";
function changed(key, value, render) {
  const stamp = JSON.stringify(value);
  if (stamps.get(key) !== stamp) {
    stamps.set(key, stamp);
    render();
  }
}
function translated() {
  document.querySelectorAll("[data-i18n]").forEach((element) => {
    element.textContent = t(element.dataset.i18n);
  });
  document.querySelectorAll("[data-placeholder]").forEach((element) => {
    element.placeholder = t(element.dataset.placeholder);
  });
  document.title = t("title");
  translateChrome();
}
function connection(online) {
  $("connection").textContent = t(online ? "connected" : "offline");
  $("connection").className = "connection " + (online ? "online" : "offline");
}
function renderState(state) {
  if (!state.ready) return;
  connection(true);
  $("operator-label").textContent = state.operator;
  $("chat-link").hidden = !state.chat_enabled;
  $("sync-time").textContent =
    t("synced") + " " + new Date().toLocaleTimeString();
  changed("services", state.services, () =>
    $("service").replaceChildren(
      ...state.services.map((service) => {
        const option = node("option", service.name);
        option.value = service.id;
        return option;
      }),
    ),
  );
  if (document.activeElement !== $("service"))
    $("service").value = state.service;
  changed("service_dates", [state.service_dates, state.service], () => {
    if (state.service_dates && $("service-date")) {
      $("service-date").replaceChildren(
        ...state.service_dates.map((d) => {
          const count = d.items > 0 ? ` (${d.items})` : "";
          const option = node("option", `${d.label}${count}`);
          option.value = d.date;
          return option;
        }),
      );
    }
  });
  if (
    $("service-date") &&
    document.activeElement !== $("service-date") &&
    state.service_date
  )
    $("service-date").value = state.service_date;
  for (const key of ["preview", "live"]) {
    const item = state[key];
    changed(key, [item, state.hidden, state.presenting], () => {
      const stage = $(key + "-stage");
      applyTheme(stage, item.theme);
      stage.firstElementChild.textContent =
        item.design ||
        item.slides[item.index] ||
        (item.title ? "" : t("no_content"));
      stage.classList.toggle(
        "hidden",
        key === "live" && (!state.presenting || state.hidden),
      );
      $(key + "-title").textContent =
        item.design || item.title || t("no_content");
      $(key + "-subtitle").textContent = item.subtitle;
      $(key + "-count").textContent = item.slides.length
        ? `${item.index + 1} / ${item.slides.length}`
        : "";
    });
  }
  $("live-status").textContent = t(
    !state.presenting ? "stopped" : state.hidden ? "hidden" : "live",
  );
  if (!$("live-viewer").getAttribute("src"))
    $("live-viewer").src = "/screen/__control_live";
  $("live-viewer").hidden = state.hidden;
  $("live-status").className = "badge " + (state.presenting ? "live" : "");
  setControlLabel(
    $("hide-toggle"),
    t(state.hidden ? "show" : "hide"),
    state.hidden ? "eye" : "eyeoff",
  );
  $("hide-toggle").classList.toggle("active", state.hidden);
  $("edit-song").hidden = !state.preview.song_id;
  $("project-preview").disabled =
    !state.preview.slides.length && !state.preview.design;
  document.querySelectorAll("[data-direction]").forEach((control) => {
    control.disabled =
      !state.presenting ||
      !state.live.slides.length ||
      (["first", "previous"].includes(control.dataset.direction)
        ? state.live.index === 0
        : state.live.index + 1 >= state.live.slides.length);
  });
  renderSlides();
  changed("playlist", state.playlist, renderPlaylist);
  changed("quick", state.quick, () => {
    $("quick-empty").hidden = !!state.quick.length;
    $("mobile-quick-items").closest("section").hidden = !state.quick.length;
    for (const id of ["quick-items", "mobile-quick-items"])
      $(id).replaceChildren(
        ...state.quick.map((item) => {
          const group = node("span", undefined, "quick-pill");
          const activate = button(item.title, async () => {
            if (await send({ action: "quick_select", id: item.id })) {
              document.dispatchEvent(
                new CustomEvent("control-quick", {
                  detail: {
                    kind:
                      {
                        Song: "song",
                        Text: "text",
                        Verse: "verse",
                        Media: "media",
                        Design: "design",
                      }[item.kind] || "song",
                  },
                }),
              );
              $("mobile-more").close();
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
  });
  changed("themes", [state.themes, state.preview.theme.id], () => {
    $("themes").replaceChildren(
      ...state.themes.map((theme) => {
        const control = button(
          "",
          (element) =>
            send(
              { action: "theme", id: theme.id, live: $("theme-live").checked },
              element,
            ),
          "theme-card" + (theme.id === state.preview.theme.id ? " active" : ""),
        );
        const swatch = node("span", t("native_font_sample"), "theme-swatch");
        swatch.dataset.i18n = "native_font_sample";
        applyTheme(swatch, theme);
        control.append(swatch, node("span", theme.name));
        control.dataset.id = theme.id;
        control.title = theme.name;
        control.ondblclick = () =>
          send({ action: "theme", id: theme.id, live: true });
        control.oncontextmenu = (event) => {
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
  $("player-panel").hidden = !state.player;
  if (state.player) {
    $("player-title").textContent = state.player.title;
    setControlLabel(
      $("player-toggle"),
      t(state.player.playing ? "pause_timer" : "play"),
      state.player.playing ? "pause" : "play",
    );
    if (document.activeElement !== $("player-seek")) {
      $("player-seek").max = state.player.duration || 1;
      $("player-seek").value = state.player.position;
    }
    if (document.activeElement !== $("player-volume"))
      $("player-volume").value = state.player.volume;
    $("player-time").textContent =
      clock(state.player.position) + " / " + clock(state.player.duration);
  }
  $("design-controls").hidden =
    !state.timers.length && !state.overlay && !state.questions;
  $("overlay-name").textContent = state.overlay || "";
  $("overlay-clear").hidden = !state.overlay;
  $("question-controls").hidden = !state.questions;
  document
    .querySelectorAll(
      '[data-design-op="start"],[data-design-op="pause"],[data-design-op="reset"]',
    )
    .forEach((control) => (control.hidden = !state.timers.length));
  changed("timers", state.timers, () =>
    $("timers").replaceChildren(
      ...state.timers.map((timer) => {
        const row = node("div", undefined, "timer");
        row.dataset.running = String(timer.running);
        row.append(
          node("span", timer.name),
          node("small", t(timer.running ? "running" : "paused")),
          node("strong", timer.text),
        );
        return row;
      }),
    ),
  );
  document.querySelector('[data-design-op="start"]').disabled =
    state.timers.every((timer) => timer.running);
  document.querySelector('[data-design-op="pause"]').disabled =
    state.timers.every((timer) => !timer.running);
  changed("screens", state.screens, () => {
    $("screens").replaceChildren(
      ...state.screens.map((screen) => {
        const link = node("a", screen.name + " ↗", "button");
        link.href = "/screen/" + encodeURIComponent(screen.slug || screen.id);
        link.target = "_blank";
        link.rel = "noopener";
        return link;
      }),
    );
    if (!state.screens.length)
      $("screens").append(node("p", t("no_screens"), "empty"));
  });
  updateChrome(state);
  highlightLibrary();
  if (catalogRevision !== state.catalog_revision) {
    catalogRevision = state.catalog_revision;
    refreshLibrary();
  }
}

function renderSlides() {
  const state = model.state,
    item = state?.[source];
  if (!item) return;
  changed(
    "slides",
    [
      source,
      model.cardsOverride ?? state.lyrics_cards,
      item.slides,
      item.index,
      state.live.title,
      state.live.subtitle,
      state.live.index,
    ],
    () => {
      const structure = JSON.stringify([source, item.slides]);
      if (
        structure === slideStructure &&
        $("slides").querySelectorAll(".slide-card").length ===
          item.slides.length
      ) {
        $("slides")
          .querySelectorAll(".slide-card")
          .forEach((card, index) => {
            card.classList.toggle("active", index === item.index);
            card.classList.toggle(
              "on-air",
              index === state.live.index &&
                item.title === state.live.title &&
                item.subtitle === state.live.subtitle,
            );
          });
        revealInList(
          $("slides"),
          $("slides").querySelector(".slide-card.active"),
        );
        return;
      }
      slideStructure = structure;
      $("slides").replaceChildren(
        ...item.slides.map((text, index) => {
          const card = node(
            "article",
            undefined,
            "slide-card" + (index === item.index ? " active" : ""),
          );
          if (
            index === state.live.index &&
            item.title === state.live.title &&
            item.subtitle === state.live.subtitle
          )
            card.classList.add("on-air");
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
      if (!item.slides.length)
        $("slides").append(
          node("p", item.design || t("select_content"), "empty"),
        );
    },
  );
}

function renderPlaylist() {
  const items = model.state.playlist;
  $("playlist-count").textContent = items.length;
  const structure = JSON.stringify(
    items.map((item) => [item.id, item.title, item.subtitle, item.kind]),
  );
  if (
    structure === playlistStructure &&
    $("playlist").querySelectorAll(".playlist-row").length === items.length
  ) {
    $("playlist")
      .querySelectorAll(".playlist-row")
      .forEach((row, index) => {
        row.classList.toggle("selected", items[index].selected);
        row.classList.toggle("played", items[index].played);
        setControlLabel(
          row.querySelector(".playlist-status"),
          t("mark_played"),
          items[index].played ? "check_circle" : "circle",
        );
      });
    const selected = items.findIndex((item) => item.selected);
    $("playlist-next").disabled =
      !items.length || selected === items.length - 1;
    $("playlist-previous").disabled = selected <= 0;
    return;
  }
  playlistStructure = structure;
  $("playlist").replaceChildren(
    ...items.map((item, index) => {
      const row = node(
        "article",
        undefined,
        "playlist-row" +
          (item.selected ? " selected" : "") +
          (item.played ? " played" : ""),
      );
      row.dataset.id = item.id;
      row.draggable = true;
      row.ondragstart = (event) => {
        event.dataTransfer.setData("application/x-control-playlist", item.id);
        event.dataTransfer.effectAllowed = "move";
      };
      row.ondragover = (event) => {
        if (
          event.dataTransfer.types.includes("application/x-control-playlist")
        ) {
          event.preventDefault();
          row.classList.add("drag-over");
        }
      };
      row.ondragleave = () => row.classList.remove("drag-over");
      row.ondrop = (event) => {
        event.preventDefault();
        row.classList.remove("drag-over");
        const id = event.dataTransfer.getData("application/x-control-playlist");
        if (id && id !== item.id)
          send({
            action: "playlist_move",
            id,
            target: item.id,
            after:
              event.clientY >
              row.getBoundingClientRect().top + row.offsetHeight / 2,
          });
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
              {
                Song: "music",
                Bible: "bible",
                Media: "media",
                Text: "text",
                Design: "design",
              }[item.kind] || "music",
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
              played: !model.state.playlist.find(
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
      const menu = (anchor) =>
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
                played: !model.state.playlist.find(
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
      row.oncontextmenu = (event) => {
        event.preventDefault();
        menu({ x: event.clientX, y: event.clientY });
      };
      const more = iconButton(t("more"), "more", menu, "native-row-more");
      row.append(select, played, more, actions);
      return row;
    }),
  );
  if (!items.length)
    $("playlist").append(node("p", t("empty_playlist"), "empty"));
  const selected = items.findIndex((item) => item.selected);
  $("playlist-next").disabled = !items.length || selected === items.length - 1;
  $("playlist-previous").disabled = selected <= 0;
}

function clock(seconds) {
  seconds = Math.max(0, Math.floor(seconds || 0));
  return Math.floor(seconds / 60) + ":" + String(seconds % 60).padStart(2, "0");
}

setupBible();
setupLibrary(openBible);
model.onState = renderState;
$("error").onclick = () => ($("error").hidden = true);
$("project-preview").onclick = async () => {
  if (await send({ action: "project_preview" }, $("project-preview")))
    showView("operation", "live");
};
function haptic(ms = 12) {
  if (typeof navigator !== "undefined" && navigator.vibrate) {
    try {
      navigator.vibrate(ms);
    } catch (_) {}
  }
}

$("hide-toggle").onclick = () => {
  haptic(16);
  send(
    { action: "visibility", hidden: !model.state?.hidden },
    $("hide-toggle"),
  );
};
$("stop").onclick = () => {
  haptic(20);
  send({ action: "stop" }, $("stop"));
};
$("service").onchange = () =>
  send({ action: "service", id: $("service").value }, $("service"));
if ($("service-date"))
  $("service-date").onchange = () =>
    send(
      { action: "service_date", date: $("service-date").value },
      $("service-date"),
    );
if ($("service-prev-date"))
  $("service-prev-date").onclick = () =>
    send({ action: "service_step_date", delta: -1 }, $("service-prev-date"));
if ($("service-next-date"))
  $("service-next-date").onclick = () =>
    send({ action: "service_step_date", delta: 1 }, $("service-next-date"));
$("overlay-clear").onclick = () =>
  send({ action: "overlay_clear" }, $("overlay-clear"));
$("playlist-clear").onclick = () =>
  send({ action: "playlist_clear" }, $("playlist-clear"));
for (const [id, offset] of [
  ["playlist-next", 1],
  ["playlist-previous", -1],
])
  $(id).onclick = () => {
    const items = model.state?.playlist || [],
      selected = items.findIndex((item) => item.selected);
    const target = items[Math.max(0, selected + offset)];
    if (target) {
      haptic(10);
      send({ action: "playlist_select", id: target.id, project: true }, $(id));
    }
  };
document
  .querySelectorAll("[data-direction]")
  .forEach(
    (control) =>
      (control.onclick = () => {
        haptic(10);
        send(
          { action: "navigate", direction: control.dataset.direction },
          control,
        );
      }),
  );
document.querySelectorAll("[data-slide-source]").forEach(
  (control) =>
    (control.onclick = () => {
      source = control.dataset.slideSource;
      document
        .querySelectorAll("[data-slide-source]")
        .forEach((item) => item.classList.toggle("active", item === control));
      renderSlides();
    }),
);
document
  .querySelectorAll("[data-design-op]")
  .forEach(
    (control) =>
      (control.onclick = () =>
        send(
          { action: "timer", operation: control.dataset.designOp },
          control,
        )),
  );
document
  .querySelectorAll(".mobile-nav button")
  .forEach(
    (control) => (control.onclick = () => showView(control.dataset.view)),
  );
document
  .querySelectorAll("[data-close]")
  .forEach(
    (control) => (control.onclick = () => control.closest("dialog").close()),
  );
$("notice-open").onclick = () => {
  $("text-dialog").querySelector(".dialog-error")?.remove();
  $("text-dialog").showModal();
  $("text-title").focus();
};
$("text-form").onsubmit = async (event) => {
  event.preventDefault();
  const intent = event.submitter?.value;
  if (
    await send(
      {
        action: "text",
        title: $("text-title").value,
        content: $("text-content").value,
        project: intent === "project",
        playlist: intent === "playlist",
      },
      event.submitter,
    )
  ) {
    $("text-dialog").close();
    if (intent === "playlist") toast(t("added"));
    else {
      document.dispatchEvent(
        new CustomEvent("control-quick", { detail: { kind: "text" } }),
      );
      showView("operation", intent === "project" ? "live" : "preview");
    }
  }
};
function syncChatTheme() {
  const frame = $("native-chat-frame") || $("chat-frame");
  try {
    if (frame?.contentDocument?.documentElement) {
      const theme = document.documentElement.dataset.theme;
      if (theme) frame.contentDocument.documentElement.dataset.theme = theme;
    }
  } catch (_) {}
}

window.syncChatTheme = syncChatTheme;

$("screens-open").onclick = () => $("screens-dialog").showModal();
$("chat-link").onclick = (event) => {
  event.preventDefault();
  if (window.setNativePage) {
    window.setNativePage("chat");
    syncChatTheme();
  } else {
    location.href = "/chat";
  }
};
$("player-toggle").onclick = () =>
  send({ action: "player", operation: "toggle" }, $("player-toggle"));
$("player-stop").onclick = () =>
  send({ action: "player", operation: "stop" }, $("player-stop"));
$("player-seek").onchange = () =>
  send({
    action: "player",
    operation: "seek",
    value: Number($("player-seek").value),
  });
$("player-volume").onchange = () =>
  send({
    action: "player",
    operation: "volume",
    value: Number($("player-volume").value),
  });
document.addEventListener("keydown", (event) => {
  if (
    event.isComposing ||
    event.target.isContentEditable ||
    /INPUT|TEXTAREA|SELECT/.test(event.target.tagName) ||
    document.querySelector("dialog[open]")
  )
    return;
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "f") {
    event.preventDefault();
    showView("library");
    document.body.dataset.searchOpen = "true";
    $("library-search").focus();
    return;
  }
  if (event.ctrlKey || event.metaKey || event.altKey || !model.online) return;
  if (event.key === "Escape" && model.state?.presenting) {
    event.preventDefault();
    send({ action: "stop" });
    return;
  }
  if (event.code === "Space" && event.target.closest("button,a")) return;
  const direction = {
    ArrowRight: "next",
    ArrowDown: "next",
    " ": "next",
    PageDown: "next",
    ArrowLeft: "previous",
    ArrowUp: "previous",
    PageUp: "previous",
    Home: "first",
    End: "last",
  }[event.key];
  if (direction && model.state?.presenting) {
    event.preventDefault();
    send({ action: "navigate", direction });
  }
  if (event.key.toLowerCase() === "h") {
    event.preventDefault();
    send({ action: "visibility", hidden: !model.state.hidden });
  }
});

function setupMobileGestures() {
  let startX = 0;
  let startY = 0;
  let startTime = 0;

  window.addEventListener(
    "touchstart",
    (event) => {
      if (event.touches.length !== 1) return;
      const touch = event.touches[0];
      const target = touch.target;
      if (
        target.closest(
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
    (event) => {
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
async function poll() {
  if (polling) return;
  polling = true;
  try {
    const state = await request("state");
    if (state.ready && language !== state.language) {
      model.texts = await request(
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
