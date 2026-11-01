import { $, model, node, send } from "/control-api.js";

const split = (direction, fraction, first, second) => ({
  Split: { direction, fraction, first, second },
});
const leaf = (name) => ({ Leaf: name });
const fallback = split(
  "Horizontal",
  0.22,
  leaf("Selection"),
  split(
    "Horizontal",
    0.38,
    leaf("Lyrics"),
    split(
      "Horizontal",
      0.54,
      split(
        "Vertical",
        0.68,
        split("Vertical", 0.5, leaf("Stage"), leaf("Preview")),
        split("Vertical", 0.5, leaf("Player"), leaf("Themes")),
      ),
      leaf("Playlist"),
    ),
  ),
);
let panels,
  root,
  parking,
  monitorTabs,
  page = "home",
  signature = "",
  renderedMobile,
  dragging = false,
  current;
const mobile = () => matchMedia("(max-width:860px)").matches;

export function setupDocking(items) {
  panels = items;
  parking = node("div");
  parking.id = "native-parking";
  parking.hidden = true;
  document.body.append(parking);
  root = $("native-layout");
  monitorTabs = $("native-monitor-tabs");
  Object.entries(panels).forEach(([kind, panel]) => {
    panel.dataset.leaf = kind;
    panel.querySelector(".panel-heading").draggable = true;
    panel.querySelector(".panel-heading").ondragstart = (event) => {
      if (mobile() || event.target.closest("button")) {
        event.preventDefault();
        return;
      }
      event.dataTransfer.setData("application/x-native-panel", kind);
    };
    panel.ondragover = (event) => {
      if (!event.dataTransfer.types.includes("application/x-native-panel"))
        return;
      event.preventDefault();
      panel.dataset.drop = dropZone(event, panel);
    };
    panel.ondragleave = () => delete panel.dataset.drop;
    panel.ondrop = (event) => {
      const source = event.dataTransfer.getData("application/x-native-panel");
      delete panel.dataset.drop;
      if (!source || source === kind) return;
      event.preventDefault();
      const layout = structuredClone(current);
      if (dropZone(event, panel) === "center") swap(layout, source, kind);
      else
        layout.root = insert(
          remove(layout.root, source),
          source,
          kind,
          dropZone(event, panel),
        );
      commit(layout);
    };
    parking.append(panel);
  });
  matchMedia("(max-width:860px)").addEventListener("change", () =>
    renderDocking(model.state, true),
  );
  document.addEventListener("control-view", () => {
    if (mobile()) renderDocking(model.state, true);
  });
}

function dropZone(event, panel) {
  const rect = panel.getBoundingClientRect();
  const x = (event.clientX - rect.left) / rect.width,
    y = (event.clientY - rect.top) / rect.height;
  if (x < 0.22) return "left";
  if (x > 0.78) return "right";
  if (y < 0.22) return "top";
  if (y > 0.78) return "bottom";
  return "center";
}
function swap(layout, a, b) {
  function visit(value) {
    if (value.Leaf)
      value.Leaf = value.Leaf === a ? b : value.Leaf === b ? a : value.Leaf;
    else {
      visit(value.Split.first);
      visit(value.Split.second);
    }
  }
  visit(layout.root);
}
function remove(value, target) {
  if (value.Leaf) return value.Leaf === target ? null : value;
  const first = remove(value.Split.first, target),
    second = remove(value.Split.second, target);
  if (!first) return second;
  if (!second) return first;
  return split(value.Split.direction, value.Split.fraction, first, second);
}
function insert(value, source, target, zone) {
  if (value.Leaf === target) {
    const direction = ["left", "right"].includes(zone)
      ? "Horizontal"
      : "Vertical";
    return ["left", "top"].includes(zone)
      ? split(direction, 0.5, leaf(source), value)
      : split(direction, 0.5, value, leaf(source));
  }
  if (value.Split) {
    value.Split.first = insert(value.Split.first, source, target, zone);
    value.Split.second = insert(value.Split.second, source, target, zone);
  }
  return value;
}

async function commit(layout) {
  const result = await send({
    action: "workspace_layout",
    bible: page === "bible",
    layout,
  });
  if (!result) renderDocking(model.state, true);
}

function build(value, path = "root") {
  if (value.Leaf) {
    const cell = node("div", undefined, "native-leaf");
    cell.append(panels[value.Leaf]);
    return cell;
  }
  const data = value.Split,
    horizontal = data.direction === "Horizontal";
  const group = node(
    "div",
    undefined,
    "native-split " + (horizontal ? "horizontal" : "vertical"),
  );
  const first = node("div", undefined, "native-first"),
    second = node("div", undefined, "native-second");
  first.style.flexBasis = data.fraction * 100 + "%";
  first.append(build(data.first, path + ".first"));
  second.append(build(data.second, path + ".second"));
  const handle = node("div", undefined, "native-splitter");
  handle.dataset.splitPath = path;
  handle.tabIndex = 0;
  handle.setAttribute("role", "separator");
  handle.setAttribute(
    "aria-orientation",
    horizontal ? "vertical" : "horizontal",
  );
  handle.onpointerdown = (event) => {
    if (event.button !== 0) return;
    dragging = true;
    handle.setPointerCapture(event.pointerId);
    document.body.classList.add("resizing");
    const rect = group.getBoundingClientRect();
    handle.onpointermove = (move) => {
      const position = horizontal
        ? (move.clientX - rect.left) / rect.width
        : (move.clientY - rect.top) / rect.height;
      data.fraction = Math.max(0.08, Math.min(0.92, position));
      first.style.flexBasis = data.fraction * 100 + "%";
    };
    const finish = () => {
      dragging = false;
      handle.onpointermove = null;
      document.body.classList.remove("resizing");
      commit(current);
    };
    handle.onpointerup = finish;
    handle.onpointercancel = finish;
  };
  handle.onkeydown = (event) => {
    if (
      !["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)
    )
      return;
    event.preventDefault();
    event.stopPropagation();
    data.fraction = Math.max(
      0.08,
      Math.min(
        0.92,
        data.fraction +
          (["ArrowRight", "ArrowDown"].includes(event.key) ? 0.02 : -0.02),
      ),
    );
    commit(current);
  };
  group.append(first, handle, second);
  return group;
}

export function setNativePage(value) {
  page = value;
  document.body.dataset.page = value;
  document.querySelector(".rail-home")?.classList.toggle("active", value === "home");
  document.querySelector('.app-rail [data-nav-kind="bible"]')?.classList.toggle("active", value === "bible");
  document.querySelector('.app-rail #chat-link')?.classList.toggle("active", value === "chat");
  renderDocking(model.state, true);
  document.dispatchEvent(new Event("control-native-page"));
}
export function resetNativeLayout() {
  return send({ action: "workspace_reset", bible: page === "bible" });
}
export function renderDocking(state, force = false) {
  if (!panels || dragging) return;
  if (page === "chat") {
    Object.values(panels).forEach((panel) => parking.append(panel));
    parking.append(monitorTabs);
    let chatPanel = $("native-chat-panel");
    if (!chatPanel) {
      chatPanel = node("div");
      chatPanel.id = "native-chat-panel";
      chatPanel.className = "native-chat-panel";
      const iframe = node("iframe");
      iframe.id = "native-chat-frame";
      iframe.src = "/chat";
      iframe.title = "Chat da Equipe";
      iframe.allow = "autoplay; microphone; camera";
      chatPanel.append(iframe);
    }
    if (!root.contains(chatPanel)) {
      root.replaceChildren(chatPanel);
    }
    return;
  }
  const chatPanel = $("native-chat-panel");
  if (chatPanel && chatPanel.parentElement) {
    parking.append(chatPanel);
  }
  const layout = page === "bible" ? state?.bible_layout : state?.layout;
  const next = JSON.stringify(layout || { root: fallback });
  if (!force && signature === next && renderedMobile === mobile()) return;
  signature = next;
  renderedMobile = mobile();
  current = structuredClone(layout || { root: fallback });
  Object.values(panels).forEach((panel) => parking.append(panel));
  parking.append(monitorTabs);
  root.replaceChildren();
  if (mobile()) {
    const view = document.body.dataset.view;
    if (view === "library") root.append(panels.Selection);
    else if (view === "playlist") root.append(panels.Playlist);
    else if (view === "themes") root.append(panels.Themes);
    else {
      root.append(monitorTabs);
      root.append(
        document.body.dataset.mobileMonitor === "preview"
          ? panels.Preview
          : panels.Stage,
      );
      root.append(panels.Lyrics);
      if (state?.player) root.append(panels.Player);
    }
  } else root.append(build(current.root));
}
