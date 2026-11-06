import { icon } from "/control-icons.js";

export const $ = (id) => document.getElementById(id);
export const model = {
  state: null,
  online: false,
  texts: {},
  onState: () => {},
  themeOverride: localStorage.getItem("control-theme"),
};
export function normalizeKey(key) {
  return String(key || "").toLowerCase().replace(/[._]/g, "-");
}

export const t = (key) => {
  if (key === undefined || key === null || key === "") return "";
  const strKey = String(key);
  if (model.texts[strKey] !== undefined) return model.texts[strKey];

  const norm = normalizeKey(strKey);
  const controlNorm = norm.startsWith("control-") ? norm : "control-" + norm;
  if (model.texts[controlNorm] !== undefined) return model.texts[controlNorm];

  if (norm.startsWith("control-")) {
    const stripped = norm.slice(8);
    if (model.texts[stripped] !== undefined) return model.texts[stripped];
  }

  const chatNorm = norm.startsWith("chat-") ? norm : "chat-" + norm;
  if (model.texts[chatNorm] !== undefined) return model.texts[chatNorm];

  if (model.texts["control." + strKey] !== undefined) return model.texts["control." + strKey];
  if (model.texts[norm] !== undefined) return model.texts[norm];

  return strKey;
};
export function node(tag, text, className) {
  const element = document.createElement(tag);
  if (text !== undefined) element.textContent = text;
  if (className) element.className = className;
  return element;
}
export function button(label, action, className = "") {
  const element = node("button", label, className);
  element.type = "button";
  element.onclick = () => action(element);
  return element;
}
export function iconButton(label, symbol, action, className = "") {
  const control = button("", action, "icon-button " + className);
  control.append(icon(symbol));
  control.title = label;
  control.setAttribute("aria-label", label);
  if (!Object.keys(model.texts).length) control.dataset.tip = label;
  return control;
}
export function setControlLabel(control, label, symbol) {
  control.title = label;
  control.setAttribute("aria-label", label);
  if (control.dataset.symbol !== symbol) {
    control.replaceChildren(icon(symbol));
    control.dataset.symbol = symbol;
  }
}
export async function request(path, options = {}) {
  const response = await fetch("/api/control/" + path, {
    ...options,
    headers: { "Content-Type": "application/json", ...options.headers },
  });
  const result = await response.json();
  if (response.status === 401 && !window.controlSigningOut) {
    window.controlSigningOut = true;
    document.body.dataset.locked = "true";
    location.reload();
  }
  if (!response.ok) throw new Error(result.error || t("failed"));
  return result;
}
export function report(error) {
  $("error").textContent = error.message || String(error);
  $("error").hidden = false;
  const dialog = document.querySelector("dialog[open]");
  if (dialog) {
    let message = dialog.querySelector(".dialog-error");
    if (!message) {
      message = node("p", undefined, "dialog-error");
      message.setAttribute("role", "alert");
      dialog.append(message);
    }
    message.textContent = error.message || String(error);
  }
}
export function accept(state) {
  if (
    state.instance === model.state?.instance &&
    state.revision < model.state?.revision
  )
    return;
  model.state = state;
  model.online = !!state.ready;
  model.onState(state);
}
let queue = Promise.resolve();
export function send(command, control) {
  if (control?.disabled) return Promise.resolve(null);
  if (control) control.disabled = true;
  const task = queue
    .then(async () => {
      if (!model.online) throw new Error(t("offline"));
      const result = await request("command", {
        method: "POST",
        body: JSON.stringify(command),
      });
      if (result.state) accept(result.state);
      $("error").hidden = true;
      return result;
    })
    .catch((error) => {
      report(error);
      return null;
    })
    .finally(() => {
      if (control) control.disabled = false;
      if (model.online && model.state) model.onState(model.state);
    });
  queue = task;
  return task;
}
export function showView(view, monitor = "preview") {
  document.body.dataset.view = view;
  if (view === "operation") document.body.dataset.mobileMonitor = monitor;
  const kind = document.querySelector(".resource-tabs .active")?.dataset.kind;
  document
    .querySelectorAll(".mobile-nav button")
    .forEach((item) =>
      item.classList.toggle(
        "active",
        item.dataset.view === view &&
          (!item.dataset.mobileKind || item.dataset.mobileKind === kind),
      ),
    );
  document
    .querySelectorAll("[data-monitor]")
    .forEach((control) =>
      control.classList.toggle(
        "active",
        control.dataset.monitor === document.body.dataset.mobileMonitor,
      ),
    );
  $("mobile-more-open")?.classList.toggle(
    "active",
    view === "library" && !["song", "bible"].includes(kind),
  );
  document.dispatchEvent(new Event("control-view"));
}
export function applyTheme(element, theme) {
  if (!theme) return;
  element.style.background = theme.image
    ? `linear-gradient(#0003,#0003),url(${JSON.stringify("/api/control/background?id=" + encodeURIComponent(theme.id))}) center/cover`
    : `linear-gradient(135deg,${theme.top},${theme.bottom})`;
  element.style.color = theme.color;
  element.style.fontFamily =
    JSON.stringify(theme.font) + ",system-ui,sans-serif";
  element.style.fontWeight = theme.weight;
  element.style.textAlign = theme.align === "justify" ? "left" : theme.align;
  element.style.textTransform =
    theme.case === "uppercase"
      ? "uppercase"
      : theme.case === "lowercase"
        ? "lowercase"
        : "none";
}
export function toast(message) {
  $("toast").textContent = message;
  $("toast").hidden = false;
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => ($("toast").hidden = true), 2600);
}
export function revealInList(container, selected) {
  if (!selected || !container.clientHeight) return;
  const bounds = container.getBoundingClientRect(),
    item = selected.getBoundingClientRect();
  if (item.top < bounds.top || item.bottom > bounds.bottom)
    container.scrollTop += item.top - bounds.top - 6;
}
