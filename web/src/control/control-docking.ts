import { $, model, node, send } from "/control-api.js";
import type { ControlState } from "../types/state.js";

export type SplitDirection = "Horizontal" | "Vertical";

export interface DockLeaf {
  Leaf: string;
}

export interface DockSplit {
  Split: {
    direction: SplitDirection;
    fraction: number;
    first: DockNode;
    second: DockNode;
  };
}

export type DockNode = DockLeaf | DockSplit;

export interface WorkspaceLayout {
  root: DockNode;
}

const split = (
  direction: SplitDirection,
  fraction: number,
  first: DockNode,
  second: DockNode,
): DockSplit => ({
  Split: { direction, fraction, first, second },
});

const leaf = (name: string): DockLeaf => ({ Leaf: name });

const fallback: DockSplit = split(
  "Horizontal",
  0.22,
  leaf("Selection"),
  split(
    "Horizontal",
    0.38,
    split("Vertical", 0.3, leaf("Player"), leaf("Lyrics")),
    split(
      "Horizontal",
      0.54,
      split(
        "Vertical",
        0.68,
        split("Vertical", 0.5, leaf("Stage"), leaf("Preview")),
        leaf("Themes"),
      ),
      leaf("Playlist"),
    ),
  ),
);

let panels: Record<string, HTMLElement>;
let root: HTMLElement;
let parking: HTMLElement;
let monitorTabs: HTMLElement;
let page = "home";
let signature = "";
let renderedMobile: boolean | undefined;
let dragging = false;
let current: WorkspaceLayout;

const mobile = (): boolean => matchMedia("(max-width:860px)").matches;

export function setupDocking(items: Record<string, HTMLElement>): void {
  panels = items;
  parking = node("div");
  parking.id = "native-parking";
  parking.hidden = true;
  document.body.append(parking);
  root = $("native-layout");
  monitorTabs = $("native-monitor-tabs");
  Object.entries(panels).forEach(([kind, panel]) => {
    panel.dataset.leaf = kind;
    const heading = panel.querySelector<HTMLElement>(".panel-heading");
    if (heading) {
      heading.draggable = true;
      heading.ondragstart = (event: DragEvent) => {
        if (
          mobile() ||
          (event.target as HTMLElement | null)?.closest("button")
        ) {
          event.preventDefault();
          return;
        }
        event.dataTransfer?.setData("application/x-native-panel", kind);
      };
    }
    panel.ondragover = (event: DragEvent) => {
      if (
        !event.dataTransfer?.types.includes("application/x-native-panel")
      )
        return;
      event.preventDefault();
      panel.dataset.drop = dropZone(event, panel);
    };
    panel.ondragleave = () => delete panel.dataset.drop;
    panel.ondrop = (event: DragEvent) => {
      const source = event.dataTransfer?.getData("application/x-native-panel");
      delete panel.dataset.drop;
      if (!source || source === kind) return;
      event.preventDefault();
      const layout = structuredClone(current);
      if (dropZone(event, panel) === "center") swap(layout, source, kind);
      else
        layout.root = insert(
          remove(layout.root, source)!,
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

function dropZone(
  event: DragEvent,
  panel: HTMLElement,
): "left" | "right" | "top" | "bottom" | "center" {
  const rect = panel.getBoundingClientRect();
  const x = (event.clientX - rect.left) / rect.width;
  const y = (event.clientY - rect.top) / rect.height;
  if (x < 0.22) return "left";
  if (x > 0.78) return "right";
  if (y < 0.22) return "top";
  if (y > 0.78) return "bottom";
  return "center";
}

function swap(layout: WorkspaceLayout, a: string, b: string): void {
  function visit(value: DockNode): void {
    if ("Leaf" in value) {
      value.Leaf = value.Leaf === a ? b : value.Leaf === b ? a : value.Leaf;
    } else {
      visit(value.Split.first);
      visit(value.Split.second);
    }
  }
  visit(layout.root);
}

function remove(value: DockNode, target: string): DockNode | null {
  if ("Leaf" in value) return value.Leaf === target ? null : value;
  const first = remove(value.Split.first, target);
  const second = remove(value.Split.second, target);
  if (!first) return second;
  if (!second) return first;
  return split(value.Split.direction, value.Split.fraction, first, second);
}

function insert(
  value: DockNode,
  source: string,
  target: string,
  zone: string,
): DockNode {
  if ("Leaf" in value && value.Leaf === target) {
    const direction: SplitDirection = ["left", "right"].includes(zone)
      ? "Horizontal"
      : "Vertical";
    return ["left", "top"].includes(zone)
      ? split(direction, 0.5, leaf(source), value)
      : split(direction, 0.5, value, leaf(source));
  }
  if ("Split" in value) {
    value.Split.first = insert(value.Split.first, source, target, zone);
    value.Split.second = insert(value.Split.second, source, target, zone);
  }
  return value;
}

async function commit(layout: WorkspaceLayout): Promise<void> {
  const result = await send({
    action: "workspace_layout",
    bible: page === "bible",
    layout,
  });
  if (!result) renderDocking(model.state, true);
}

function build(value: DockNode, path = "root"): HTMLElement {
  if ("Leaf" in value) {
    const cell = node("div", undefined, "native-leaf");
    if (panels[value.Leaf]) cell.append(panels[value.Leaf]);
    return cell;
  }
  const data = value.Split;
  const horizontal = data.direction === "Horizontal";
  const group = node(
    "div",
    undefined,
    "native-split " + (horizontal ? "horizontal" : "vertical"),
  );
  const first = node("div", undefined, "native-first");
  const second = node("div", undefined, "native-second");
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
  handle.onpointerdown = (event: PointerEvent) => {
    if (event.button !== 0) return;
    dragging = true;
    handle.setPointerCapture(event.pointerId);
    document.body.classList.add("resizing");
    const rect = group.getBoundingClientRect();
    handle.onpointermove = (move: PointerEvent) => {
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
  handle.onkeydown = (event: KeyboardEvent) => {
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

export function setNativePage(value: string): void {
  page = value;
  document.body.dataset.page = value;
  document
    .querySelector(".rail-home")
    ?.classList.toggle("active", value === "home");
  document
    .querySelector('.app-rail [data-nav-kind="bible"]')
    ?.classList.toggle("active", value === "bible");
  document
    .querySelector(".app-rail #chat-link")
    ?.classList.toggle("active", value === "chat");
  renderDocking(model.state, true);
  document.dispatchEvent(new Event("control-native-page"));
}

export function resetNativeLayout(): Promise<unknown> {
  return send({ action: "workspace_reset", bible: page === "bible" });
}

export function renderDocking(state: ControlState | null, force = false): void {
  if (!panels || dragging) return;
  if (page === "chat") {
    Object.values(panels).forEach((panel) => parking.append(panel));
    if (monitorTabs) parking.append(monitorTabs);
    let chatPanel = $<HTMLElement>("native-chat-panel");
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
  const rawLayout = page === "bible" ? state?.bible_layout : state?.layout;
  const layout = (typeof rawLayout === "object" && rawLayout !== null ? rawLayout : undefined) as WorkspaceLayout | undefined;
  const next = JSON.stringify(layout || { root: fallback });
  if (!force && signature === next && renderedMobile === mobile()) return;
  signature = next;
  renderedMobile = mobile();
  current = structuredClone(layout || { root: fallback });
  Object.values(panels).forEach((panel) => parking.append(panel));
  if (monitorTabs) parking.append(monitorTabs);
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
  } else {
    root.append(build(current.root));
  }
}

