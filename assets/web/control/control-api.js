import { icon } from "/control-icons.js";
export const $ = (id) => document.getElementById(id);
export const model = {
    state: null,
    online: false,
    texts: {},
    onState: () => { },
    themeOverride: localStorage.getItem("control-theme"),
};
export function normalizeKey(key) {
    return String(key || "")
        .toLowerCase()
        .replace(/[._]/g, "-");
}
export const t = (key) => {
    if (key === undefined || key === null || key === "")
        return "";
    const strKey = String(key);
    if (model.texts[strKey] !== undefined)
        return model.texts[strKey];
    const norm = normalizeKey(strKey);
    const controlNorm = norm.startsWith("control-") ? norm : "control-" + norm;
    if (model.texts[controlNorm] !== undefined)
        return model.texts[controlNorm];
    if (norm.startsWith("control-")) {
        const stripped = norm.slice(8);
        if (model.texts[stripped] !== undefined)
            return model.texts[stripped];
    }
    const chatNorm = norm.startsWith("chat-") ? norm : "chat-" + norm;
    if (model.texts[chatNorm] !== undefined)
        return model.texts[chatNorm];
    if (model.texts["control." + strKey] !== undefined)
        return model.texts["control." + strKey];
    if (model.texts[norm] !== undefined)
        return model.texts[norm];
    return strKey;
};
export function node(tag, text, className) {
    const element = document.createElement(tag);
    if (text !== undefined)
        element.textContent = text;
    if (className)
        element.className = className;
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
    if (!Object.keys(model.texts).length)
        control.dataset.tip = label;
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
    if (!response.ok)
        throw new Error(result.error || t("failed"));
    return result;
}
export function report(error) {
    const msg = error instanceof Error ? error.message : String(error);
    const errEl = $("error");
    if (errEl) {
        errEl.textContent = msg;
        errEl.hidden = false;
    }
    const dialog = document.querySelector("dialog[open]");
    if (dialog) {
        let message = dialog.querySelector(".dialog-error");
        if (!message) {
            message = node("p", undefined, "dialog-error");
            message.setAttribute("role", "alert");
            dialog.append(message);
        }
        message.textContent = msg;
    }
}
export function accept(state) {
    const cur = model.state;
    if (cur &&
        state.instance === cur.instance &&
        state.revision !== undefined &&
        cur.revision !== undefined &&
        state.revision < cur.revision) {
        return;
    }
    model.state = state;
    model.online = !!state.ready;
    model.onState(state);
}
let queue = Promise.resolve();
export function send(command, control) {
    if (control?.disabled)
        return Promise.resolve(null);
    if (control)
        control.disabled = true;
    const task = queue
        .then(async () => {
        if (!model.online)
            throw new Error(t("offline"));
        const result = await request("command", {
            method: "POST",
            body: JSON.stringify(command),
        });
        if (result.state)
            accept(result.state);
        const errEl = $("error");
        if (errEl)
            errEl.hidden = true;
        return result;
    })
        .catch((error) => {
        report(error);
        return null;
    })
        .finally(() => {
        if (control)
            control.disabled = false;
        if (model.online && model.state)
            model.onState(model.state);
    });
    queue = task;
    return task;
}
export function showView(view, monitor = "preview") {
    document.body.dataset.view = view;
    if (view === "operation")
        document.body.dataset.mobileMonitor = monitor;
    const activeTab = document.querySelector(".resource-tabs .active");
    const kind = activeTab?.dataset.kind;
    document
        .querySelectorAll(".mobile-nav button")
        .forEach((item) => item.classList.toggle("active", item.dataset.view === view &&
        (!item.dataset.mobileKind || item.dataset.mobileKind === kind)));
    document
        .querySelectorAll("[data-monitor]")
        .forEach((control) => control.classList.toggle("active", control.dataset.monitor === document.body.dataset.mobileMonitor));
    const mobileMore = $("mobile-more-open");
    if (mobileMore) {
        mobileMore.classList.toggle("active", view === "library" && (!kind || !["song", "bible"].includes(kind)));
    }
    document.dispatchEvent(new Event("control-view"));
}
let shaderGl = null;
let shaderProg = null;
const shaderLocs = {};
let shaderCanvas = null;
const themeThumbCache = new Map();
async function initShaderRenderer() {
    if (shaderGl && shaderProg)
        return true;
    try {
        shaderCanvas = document.createElement("canvas");
        shaderCanvas.width = 160;
        shaderCanvas.height = 90;
        shaderGl = shaderCanvas.getContext("webgl2", {
            alpha: false,
            antialias: false,
            preserveDrawingBuffer: true,
        });
        if (!shaderGl)
            return false;
        const vertex = "#version 300 es\nprecision highp float;out vec2 uv;void main(){vec2 p=vec2(float((gl_VertexID<<1)&2),float(gl_VertexID&2));gl_Position=vec4(p*vec2(2,-2)+vec2(-1,1),0,1);uv=p;}";
        const resp = await fetch("/shader.glsl");
        if (!resp.ok)
            return false;
        const fragment = await resp.text();
        const compile = (type, src) => {
            if (!shaderGl)
                return null;
            const s = shaderGl.createShader(type);
            if (!s)
                return null;
            shaderGl.shaderSource(s, src);
            shaderGl.compileShader(s);
            return s;
        };
        const vs = compile(shaderGl.VERTEX_SHADER, vertex);
        const fs = compile(shaderGl.FRAGMENT_SHADER, fragment);
        if (!vs || !fs)
            return false;
        shaderProg = shaderGl.createProgram();
        if (!shaderProg)
            return false;
        shaderGl.attachShader(shaderProg, vs);
        shaderGl.attachShader(shaderProg, fs);
        shaderGl.linkProgram(shaderProg);
        const vao = shaderGl.createVertexArray();
        if (vao)
            shaderGl.bindVertexArray(vao);
        for (const name of [
            "screen",
            "motion",
            "detail",
            "color0",
            "color1",
            "color2",
            "finish",
        ]) {
            shaderLocs[name] = shaderGl.getUniformLocation(shaderProg, name);
        }
        return true;
    }
    catch {
        return false;
    }
}
export async function applyThemeBackground(element, theme) {
    if (!element || !theme)
        return;
    if (theme.image) {
        element.style.background = `linear-gradient(#0003,#0003),url(${JSON.stringify("/api/control/background?id=" + encodeURIComponent(theme.id))}) center/cover`;
        return;
    }
    // Pre-set baseline gradient immediately so element is never blank
    if (theme.bg_type === "color" || theme.bg_type === "solid") {
        element.style.background = theme.top;
    }
    else if (Array.isArray(theme.colors) && theme.colors.length >= 2) {
        element.style.background = `linear-gradient(135deg, ${theme.colors.join(", ")})`;
    }
    else {
        element.style.background = `linear-gradient(135deg,${theme.top},${theme.bottom})`;
    }
    if (theme.shader && theme.shader_options) {
        const cacheKey = `${theme.id}_${theme.shader_index}_${JSON.stringify(theme.shader_options)}`;
        if (themeThumbCache.has(cacheKey)) {
            element.style.background = `linear-gradient(#0002,#0002),url("${themeThumbCache.get(cacheKey)}") center/cover`;
            return;
        }
        const ready = await initShaderRenderer();
        if (ready && shaderGl && shaderProg && shaderCanvas) {
            try {
                const p = theme.shader_options;
                const gl = shaderGl;
                gl.viewport(0, 0, shaderCanvas.width, shaderCanvas.height);
                gl.useProgram(shaderProg);
                const time = 2.0;
                gl.uniform4f(shaderLocs.screen, shaderCanvas.width, shaderCanvas.height, time, theme.shader_index || 0);
                gl.uniform4f(shaderLocs.motion, p.speed ?? 0.2, p.scale ?? 1, p.intensity ?? 1, p.softness ?? 0.5);
                gl.uniform4f(shaderLocs.detail, p.detail ?? 4, ((p.direction ?? 0) * Math.PI) / 180, p.seed ?? 0, p.vignette ?? 0.4);
                const colors = Array.isArray(p.colors) && p.colors.length >= 3
                    ? p.colors
                    : [
                        [6, 12, 28],
                        [24, 160, 130],
                        [90, 60, 180],
                    ];
                colors.forEach((c, i) => gl.uniform4f(shaderLocs["color" + i], c[0] / 255, c[1] / 255, c[2] / 255, 1));
                gl.uniform4f(shaderLocs.finish, p.grain ?? 0.01, 0, 0, 0);
                gl.drawArrays(gl.TRIANGLES, 0, 3);
                const dataUrl = shaderCanvas.toDataURL("image/jpeg", 0.85);
                themeThumbCache.set(cacheKey, dataUrl);
                element.style.background = `linear-gradient(#0002,#0002),url("${dataUrl}") center/cover`;
            }
            catch {
                /* Fallback gradient already set */
            }
        }
    }
}
export function applyTheme(element, theme) {
    if (!element || !theme)
        return;
    applyThemeBackground(element, theme);
    element.style.color = theme.color;
    element.style.fontFamily =
        JSON.stringify(theme.font) + ",system-ui,sans-serif";
    if (theme.weight !== undefined) {
        element.style.fontWeight = String(theme.weight);
    }
    element.style.textAlign = theme.align === "justify" ? "left" : (theme.align || "center");
    element.style.textTransform =
        theme.case === "uppercase"
            ? "uppercase"
            : theme.case === "lowercase"
                ? "lowercase"
                : "none";
}
export const toast = Object.assign((message) => {
    const toastEl = $("toast");
    if (!toastEl)
        return;
    toastEl.textContent = message;
    toastEl.hidden = false;
    clearTimeout(toast.timer);
    toast.timer = setTimeout(() => {
        toastEl.hidden = true;
    }, 2600);
}, { timer: undefined });
export function revealInList(container, selected) {
    if (!container || !selected || !container.clientHeight)
        return;
    const bounds = container.getBoundingClientRect();
    const item = selected.getBoundingClientRect();
    if (item.top < bounds.top || item.bottom > bounds.bottom) {
        container.scrollTop += item.top - bounds.top - 6;
    }
}
