import type { DesignRenderer, DesignScene } from "./design.js";

const byId = <T extends HTMLElement = HTMLElement>(id: string): T =>
  document.getElementById(id) as T;

const stage = byId("stage");
const box = byId("box");
const lyrics = byId("lyrics");
const background = byId("background");
const video = byId<HTMLVideoElement>("video");
const canvas = byId<HTMLCanvasElement>("shader");
const layer = byId("current-layer");
const nativeText = document.createElement("img");
nativeText.id = "native-text";
nativeText.alt = "";
nativeText.hidden = true;
Object.assign(nativeText.style, { position: "absolute", inset: "0", width: "100%", height: "100%", pointerEvents: "none" });
layer.append(nativeText);
const layers = byId("text-layers");
const title = byId("title");
const connection = byId("connection");
const backgroundError = byId("background-error");
const stageClock = byId("stage-clock");
const nextPreview = byId("next-preview");

const designLayer = document.createElement("canvas");
designLayer.id = "design-layers";
designLayer.hidden = true;
Object.assign(designLayer.style, {
  position: "absolute",
  inset: "0",
  width: "100%",
  height: "100%",
  pointerEvents: "none",
  zIndex: "3",
});
stage.append(designLayer);

let designRenderer: DesignRenderer | null = null;
import("/design.js")
  .then(({ DesignRenderer }: { DesignRenderer: new (c: HTMLCanvasElement) => DesignRenderer }) => {
    designRenderer = new DesignRenderer(designLayer);
    if (current) {
      designRenderer.setScene(current.design ?? null);
      fit();
    }
  })
  .catch((error) => console.error("Não foi possível carregar o design:", error));

const screenId = location.pathname.split("/").filter(Boolean).pop() || "";
let last = "";
let current: ViewerState | null = null;
let failures = 0;
let fontUrl = "";
let pendingFont = 0;
let shaderState: ViewerState | null = null;
let shaderSync = 0;
let gl: WebGL2RenderingContext | null = null;
let program: WebGLProgram | null = null;
const locations: Record<string, WebGLUniformLocation | null> = {};
let activeAnimations: Animation[] = [];
let shaderStarting = false;
let shaderSource = "";
let failedShader = "";
let shaderLoopStarted = false;
let shaderVertexArray: WebGLVertexArrayObject | null = null;

export interface ViewerOptions {
  auto_fit?: boolean;
  wrap_text?: boolean;
  text_opacity?: number;
  text_glow?: number;
  paragraph_spacing?: number;
  text_rotation?: number;
  transition_ms?: number;
  transition_strength?: number;
  transition_easing?: string;
  background_overlay?: number;
  background_brightness?: number;
  video?: {
    start?: number;
    repeat?: boolean;
    speed?: number;
    fit?: string;
    mirror?: boolean;
    saturation?: number;
    contrast?: number;
  };
  shader?: {
    speed?: number;
    scale?: number;
    intensity?: number;
    softness?: number;
    detail?: number;
    direction?: number;
    seed?: number;
    vignette?: number;
    colors?: [number, number, number][];
    grain?: number;
  };
}

export interface ViewerState {
  nativeText?: boolean;
  textLayer?: string | null;
  showTitle?: boolean;
  aspect?: number;
  box: [number, number, number, number];
  vertical?: string;
  boxBackground?: string;
  boxBorder?: string;
  radius: number;
  padding: number;
  letterSpacing: number;
  shadow?: string;
  color?: string;
  outline?: string;
  size: number;
  transition?: string;
  motion?: boolean;
  hidden?: boolean;
  text?: string;
  nextText?: string;
  showNextPreview?: boolean;
  showComments?: boolean;
  design?: DesignScene | null;
  name?: string;
  transparent?: boolean;
  background?: string;
  title?: string;
  showClock?: boolean;
  lineHeight?: string;
  weight?: string | number;
  italic?: boolean;
  underline?: boolean;
  strike?: boolean;
  align?: CanvasTextAlign;
  case?: string;
  font?: string;
  fontUrl?: string;
  backgroundType?: string;
  videoUrl?: string;
  shaderFragment?: string;
  clock?: number;
  shaderIndex?: number;
  options?: ViewerOptions;
}

function updateClock(): void {
  if (stageClock && !stageClock.hidden) {
    const d = new Date();
    stageClock.textContent =
      d.getHours().toString().padStart(2, "0") +
      ":" +
      d.getMinutes().toString().padStart(2, "0");
  }
}
setInterval(updateClock, 1000);

const fullscreenBtn = byId("fullscreen");
if (fullscreenBtn) {
  fullscreenBtn.onclick = async () => {
    try {
      if (document.fullscreenElement) await document.exitFullscreen();
      else await document.documentElement.requestFullscreen();
    } catch {
      connection.textContent =
        "Use o modo de tela cheia do navegador (F11).";
      connection.hidden = false;
    }
  };
}

function options(s: ViewerState): Required<ViewerOptions> {
  return {
    auto_fit: true,
    wrap_text: true,
    text_opacity: 1,
    text_glow: 0,
    paragraph_spacing: 0,
    text_rotation: 0,
    transition_ms: 400,
    transition_strength: 12,
    transition_easing: "smooth",
    background_overlay: 0,
    background_brightness: 1,
    video: {},
    shader: {},
    ...s.options,
  };
}

function scalePixels(css: string, scale: number): string {
  return css.replace(/(-?[\d.]+)px/g, (_, n) => Number(n) * scale + "px");
}

function fit(): void {
  if (!current) return;
  const s = current,
    o = options(s);
  let w = innerWidth,
    h = innerHeight;
  if (s.aspect && s.aspect > 0) {
    w = Math.min(w, h * s.aspect);
    h = w / s.aspect;
  }
  stage.style.width = w + "px";
  stage.style.height = h + "px";
  const scale = h / 720,
    b = s.box || [0, 0, 1, 1];
  Object.assign(box.style, {
    left: b[0] * 100 + "%",
    top: b[1] * 100 + "%",
    width: b[2] * 100 + "%",
    height: b[3] * 100 + "%",
    alignItems:
      ({ top: "flex-start", bottom: "flex-end" } as Record<string, string>)[
        s.vertical || ""
      ] || "center",
    background: s.boxBackground || "transparent",
    border: scalePixels(s.boxBorder || "none", scale),
    borderRadius: s.radius * scale + "px",
    padding: s.padding * scale + "px",
  });
  lyrics.style.letterSpacing = s.letterSpacing * scale + "px";
  const shadows =
    s.shadow && s.shadow !== "none" ? [scalePixels(s.shadow, scale)] : [];
  if (o.text_glow > 0) shadows.push(`0 0 ${o.text_glow * scale}px ${s.color}`);
  lyrics.style.textShadow = shadows.join(",") || "none";
  lyrics.style.webkitTextStroke = scalePixels(
    s.outline || "0 transparent",
    scale,
  );
  lyrics
    .querySelectorAll<HTMLElement>(".paragraph")
    .forEach(
      (p, i, all) =>
        (p.style.marginBottom =
          i + 1 < all.length ? o.paragraph_spacing * scale + "px" : "0"),
    );
  let size = Math.max(12, s.size * scale);
  const availableHeight = box.clientHeight - 2 * s.padding * scale;
  const availableWidth = box.clientWidth - 2 * s.padding * scale;
  const measure = (n: number) => {
    lyrics.style.fontSize = n + "px";
    return (
      lyrics.scrollHeight <= availableHeight + 1 &&
      lyrics.scrollWidth <= availableWidth + 1
    );
  };
  if (availableHeight > 20 && availableWidth > 20) {
    if (!measure(size) && o.auto_fit) {
      let low = 12,
        high = size;
      for (let i = 0; i < 18; i++) {
        const mid = (low + high) / 2;
        if (measure(mid)) low = mid;
        else high = mid;
      }
      size = Math.max(12, low);
    }
  }
  lyrics.style.fontSize = size + "px";
  const resolutionScale = Math.min(
    devicePixelRatio || 1,
    1.5,
    1920 / w,
    1080 / h,
  );
  canvas.width = Math.max(1, Math.round(w * resolutionScale));
  canvas.height = Math.max(1, Math.round(h * resolutionScale));
  designRenderer?.resize(w, h);
}

function transition(oldLayer: HTMLElement | null, s: ViewerState): void {
  const o = options(s),
    effect = s.transition || "fade";
  activeAnimations.forEach((a) => a.cancel());
  activeAnimations = [];
  if (
    !oldLayer ||
    !s.motion ||
    effect === "cut" ||
    s.hidden ||
    !s.text ||
    matchMedia("(prefers-reduced-motion: reduce)").matches
  ) {
    oldLayer?.remove();
    return;
  }
  const scale = stage.clientHeight / 720,
    blur = o.transition_strength * scale;
  const incoming: Keyframe[] = [{ opacity: 0 }, { opacity: 1 }];
  const outgoing: Keyframe[] = [{ opacity: 1 }, { opacity: 0 }];
  if (effect === "blur") {
    incoming[0].filter = `blur(${blur}px)`;
    incoming[1].filter = "blur(0px)";
    outgoing[0].filter = "blur(0px)";
    outgoing[1].filter = `blur(${blur}px)`;
  }
  const axes: Record<string, [string, string, string]> = {
    slide_left: ["X", "100%", "-100%"],
    slide_right: ["X", "-100%", "100%"],
    slide_up: ["Y", "100%", "-100%"],
    slide_down: ["Y", "-100%", "100%"],
  };
  if (axes[effect]) {
    const [axis, start, end] = axes[effect];
    incoming[0].transform = `translate${axis}(${start})`;
    incoming[1].transform = `translate${axis}(0)`;
    outgoing[0].transform = `translate${axis}(0)`;
    outgoing[1].transform = `translate${axis}(${end})`;
  }
  if (effect === "zoom" || effect === "cube") {
    incoming[0].transform = "scale(.84)";
    incoming[1].transform = "scale(1)";
    outgoing[0].transform = "scale(1)";
    outgoing[1].transform = "scale(1.16)";
  }
  if (effect === "wipe") {
    incoming[0].clipPath = "inset(0 100% 0 0)";
    incoming[1].clipPath = "inset(0 0 0 0)";
    outgoing[0].clipPath = "inset(0 0 0 0)";
    outgoing[1].clipPath = "inset(0 0 0 100%)";
  }
  const timing: KeyframeAnimationOptions = {
    duration: o.transition_ms,
    easing:
      ({
        linear: "linear",
        ease_out: "cubic-bezier(.2,.8,.2,1)",
      } as Record<string, string>)[o.transition_easing] || "ease-in-out",
  };
  activeAnimations.push(layer.animate(incoming, timing));
  const oldAnimation = oldLayer.animate(outgoing, timing);
  activeAnimations.push(oldAnimation);
  oldAnimation.finished
    .then(() => oldLayer.remove())
    .catch(() => oldLayer.remove());
}

async function loadFont(s: ViewerState): Promise<void> {
  if (!s.fontUrl || s.fontUrl === fontUrl) return;
  fontUrl = s.fontUrl;
  const ticket = ++pendingFont;
  try {
    const face = await new FontFace(
      "omni-live-" + ticket,
      `url(${JSON.stringify(s.fontUrl)})`,
      { weight: String(s.weight), style: s.italic ? "italic" : "normal" },
    ).load();
    if (ticket !== pendingFont) return;
    document.fonts.add(face);
    lyrics.style.fontFamily =
      JSON.stringify(face.family) + ",Arial,sans-serif";
    fit();
  } catch {
    lyrics.style.fontFamily = JSON.stringify(s.font) + ",Arial,sans-serif";
  }
}

function isDirectiveLine(text: string): boolean {
  const t = text.trim();
  if (t.startsWith("#*")) return true;
  if (!t.startsWith("//")) return false;
  const after = t.slice(2).trimStart();
  return /^(f|fonte|font|c|cor|color|t|s|tamanho|size|sombra|shadow|fundo|box|b|a|alinhar|align|l|linha|line|ls|letra|letter)(:|\s)/i.test(
    after,
  );
}

function appendFormattedRuns(parent: HTMLElement, text: string): void {
  const regex =
    /(\[(c|cor|color):\s*([^\]]+)\]([\s\S]*?)\[\/(?:c|cor|color)\]|\[(t|s|tamanho|size):\s*([^\]]+)\]([\s\S]*?)\[\/(?:t|s|tamanho|size)\]|\[(f|fonte|font):\s*([^\]]+)\]([\s\S]*?)\[\/(?:f|fonte|font)\]|\[(b|box|sombra|shadow|fundo):\s*([^\]]+)\]([\s\S]*?)\[\/(?:b|box|sombra|shadow|fundo)\]|\[(a|alinhar|align):\s*([^\]]+)\]([\s\S]*?)\[\/(?:a|alinhar|align)\]|\*\*([\s\S]*?)\*\*|\*([^*]+?)\*|__([\s\S]*?)__|~~([\s\S]*?)~~|\[\/?[a-zA-Z]+(?::[^\]]*)?\])/gi;

  let lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parent.append(
        document.createTextNode(text.slice(lastIndex, match.index)),
      );
    }
    if (match[2]) {
      const span = document.createElement("span");
      span.style.color = match[3].trim();
      appendFormattedRuns(span, match[4]);
      parent.append(span);
    } else if (match[5]) {
      const span = document.createElement("span");
      appendFormattedRuns(span, match[7]);
      parent.append(span);
    } else if (match[8]) {
      const span = document.createElement("span");
      span.style.fontFamily =
        JSON.stringify(match[9].trim()) + ",Arial,sans-serif";
      appendFormattedRuns(span, match[10]);
      parent.append(span);
    } else if (match[11]) {
      const span = document.createElement("span");
      appendFormattedRuns(span, match[13]);
      parent.append(span);
    } else if (match[14]) {
      const span = document.createElement("span");
      appendFormattedRuns(span, match[16]);
      parent.append(span);
    } else if (match[17] !== undefined) {
      const el = document.createElement("strong");
      appendFormattedRuns(el, match[17]);
      parent.append(el);
    } else if (match[18] !== undefined) {
      const el = document.createElement("em");
      appendFormattedRuns(el, match[18]);
      parent.append(el);
    } else if (match[19] !== undefined) {
      const el = document.createElement("u");
      appendFormattedRuns(el, match[19]);
      parent.append(el);
    } else if (match[20] !== undefined) {
      const el = document.createElement("s");
      appendFormattedRuns(el, match[20]);
      parent.append(el);
    }
    lastIndex = regex.lastIndex;
  }
  if (lastIndex < text.length) {
    parent.append(document.createTextNode(text.slice(lastIndex)));
  }
}

function renderStrophe(
  container: HTMLElement,
  rawText: string | undefined,
  isNext: boolean,
  showComments?: boolean,
  maxLines = 0,
): boolean {
  if (!rawText) return false;
  const lines = rawText.split("\n");
  let count = 0;
  for (const line of lines) {
    if (isDirectiveLine(line)) continue;
    const trimmed = line.trim();
    if (!showComments && trimmed.startsWith("//")) continue;
    if (isNext && !trimmed) continue;
    if (maxLines > 0 && count >= maxLines) break;
    count++;
    const p = document.createElement("span");
    p.className = "paragraph" + (isNext ? " next-paragraph" : "");
    if (trimmed.startsWith("//")) {
      p.classList.add("comment");
      p.textContent = trimmed;
    } else if (line.includes("//")) {
      const commentIdx = line.indexOf("//");
      const mainPart = line.slice(0, commentIdx);
      const commentPart = line.slice(commentIdx);
      appendFormattedRuns(p, mainPart);
      if (showComments && !isDirectiveLine(commentPart)) {
        const cSpan = document.createElement("span");
        cSpan.className = "paragraph comment";
        cSpan.textContent = " " + commentPart.trim();
        p.append(cSpan);
      }
    } else {
      appendFormattedRuns(p, line || "\u200b");
    }
    container.append(p);
  }
  return count > 0;
}

function render(s: ViewerState): void {
  const o = options(s);
  const changed =
    !current ||
    current.textLayer !== s.textLayer ||
    current.text !== s.text ||
    current.nextText !== s.nextText ||
    current.showNextPreview !== s.showNextPreview ||
    current.hidden !== s.hidden;
  let oldLayer: HTMLElement | null = null;
  if (changed) {
    layers
      .querySelectorAll(".previous-layer")
      .forEach((el) => el.remove());
    if (current?.text && !s.hidden) {
      oldLayer = layer.cloneNode(true) as HTMLElement;
      oldLayer.removeAttribute("id");
      oldLayer.classList.add("previous-layer");
      oldLayer
        .querySelectorAll("[id]")
        .forEach((el) => el.removeAttribute("id"));
      layers.prepend(oldLayer);
    }
    lyrics.replaceChildren();

    // 1. Estrofe atual
    if (!s.nativeText) renderStrophe(lyrics, s.text, false, s.showComments);

    // 2. Estrofe a seguir (junto com o texto da música, em cor mais fraca, limitando às duas primeiras linhas)
    if (s.showNextPreview && s.nextText && s.nextText.trim()) {
      const nextDiv = document.createElement("div");
      nextDiv.className = "next-strophe";
      const hasNext = renderStrophe(
        nextDiv,
        s.nextText,
        true,
        s.showComments,
        2,
      );
      if (hasNext) {
        lyrics.append(nextDiv);
      }
    }
  }
  current = s;
  designLayer.hidden = !!s.hidden || !s.design;
  designRenderer?.setScene(s.design ?? null);
  document.title = (s.name || "") + " · Omni Presenter";
  document.body.style.background = s.transparent ? "transparent" : "#000";
  // Hide content, not the theme background, just like the physical projector.
  background.style.background = s.background || "transparent";
  box.style.display = s.nativeText || s.hidden || !s.text ? "none" : "flex";
  const nativeUrl = s.nativeText && !s.hidden ? s.textLayer : null;
  if (nativeUrl && nativeText.dataset.source !== nativeUrl) {
    nativeText.hidden = true;
    nativeText.dataset.source = nativeUrl;
    nativeText.onload = () => {
      if (current?.textLayer === nativeUrl && !current.hidden) {
        nativeText.hidden = false;
        nativeText.dataset.ready = nativeUrl;
      }
    };
    nativeText.onerror = () => { delete nativeText.dataset.source; };
    nativeText.src = nativeUrl;
  } else nativeText.hidden = !nativeUrl || nativeText.dataset.ready !== nativeUrl;
  title.textContent = s.title || "";
  title.hidden = !!s.nativeText && !s.showTitle;
  if (stageClock) {
    stageClock.hidden = !s.showClock;
    if (s.showClock) updateClock();
  }
  if (nextPreview) {
    nextPreview.hidden = true;
  }
  lyrics.style.whiteSpace = "normal";
  lyrics
    .querySelectorAll<HTMLElement>(".paragraph")
    .forEach((p) => (p.style.whiteSpace = o.wrap_text ? "pre-wrap" : "pre"));
  lyrics.style.opacity = String(o.text_opacity);
  lyrics.style.lineHeight = s.lineHeight || "1.2";
  lyrics.style.color = s.color || "#fff";
  lyrics.style.fontWeight = String(s.weight || 400);
  lyrics.style.fontStyle = s.italic ? "italic" : "normal";
  lyrics.style.textDecoration = [
    s.underline ? "underline" : "",
    s.strike ? "line-through" : "",
  ].join(" ");
  lyrics.style.textAlign = s.align || "center";
  lyrics.style.textTransform =
    ({
      uppercase: "uppercase",
      lowercase: "lowercase",
      titlecase: "capitalize",
    } as Record<string, string>)[s.case || ""] || "none";
  lyrics.style.transform = `rotate(${o.text_rotation}deg)`;
  lyrics.style.transformOrigin =
    ({ left: "0", right: "100%" } as Record<string, string>)[s.align || ""] ||
    "50%";
  loadFont(s);
  fit();
  requestAnimationFrame(fit);
  const showBackground = !s.transparent;
  const showVideo =
    showBackground && s.backgroundType === "video" && !!s.videoUrl;
  video.style.display = showVideo ? "block" : "none";
  if (showVideo && s.videoUrl) {
    if (video.dataset.source !== s.videoUrl) {
      video.dataset.source = s.videoUrl;
      video.src = s.videoUrl;
      video.onloadedmetadata = () => {
        video.currentTime = Math.min(
          o.video.start || 0,
          video.duration || 0,
        );
        video.play().catch(() => {});
      };
    }
    video.loop = o.video.repeat !== false;
    video.playbackRate = o.video.speed || 1;
    video.style.objectFit = o.video.fit || "cover";
    video.style.transform = o.video.mirror ? "scaleX(-1)" : "none";
    video.style.filter = `saturate(${o.video.saturation ?? 1}) contrast(${o.video.contrast ?? 1})`;
    if (video.paused && !video.ended) video.play().catch(() => {});
  } else {
    video.pause();
  }
  const brightness = o.background_brightness;
  byId("brightness").style.background = !showBackground
    ? "transparent"
    : brightness < 1
      ? `rgba(0,0,0,${1 - brightness})`
      : `rgba(255,255,255,${Math.min(1, (brightness - 1) * 0.35)})`;
  byId("dimmer").style.background = showBackground
    ? `rgba(0,0,0,${o.background_overlay})`
    : "transparent";
  if (showBackground && s.backgroundType === "shader") {
    shaderState = s;
    shaderSync = performance.now();
    canvas.style.display = "block";
    const source = s.shaderFragment || "builtin";
    if (
      (!gl || shaderSource !== source) &&
      failedShader !== source &&
      !shaderStarting
    ) {
      startShader(source);
    }
  } else {
    shaderState = null;
    canvas.style.display = "none";
  }
  if (changed) transition(oldLayer, s);
  if (s.hidden) {
    activeAnimations.forEach((a) => a.cancel());
    layers
      .querySelectorAll(".previous-layer")
      .forEach((el) => el.remove());
  }
}

async function startShader(source: string): Promise<void> {
  shaderStarting = true;
  const shaders: WebGLShader[] = [];
  let nextProgram: WebGLProgram | null = null;
  try {
    gl ||= canvas.getContext("webgl2", { alpha: false, antialias: false });
    if (!gl) throw Error("WebGL 2 indisponível neste navegador.");
    const compile = (type: number, src: string) => {
      const shader = gl!.createShader(type)!;
      shaders.push(shader);
      gl!.shaderSource(shader, src);
      gl!.compileShader(shader);
      if (!gl!.getShaderParameter(shader, gl!.COMPILE_STATUS)) {
        throw Error(gl!.getShaderInfoLog(shader) || "Shader error");
      }
      return shader;
    };
    const fragment =
      source === "builtin" ? await (await fetch("/shader.glsl")).text() : source;
    const vertex =
      "#version 300 es\nprecision highp float;out vec2 uv;void main(){vec2 p=vec2(float((gl_VertexID<<1)&2),float(gl_VertexID&2));gl_Position=vec4(p*vec2(2,-2)+vec2(-1,1),0,1);uv=p;}";
    nextProgram = gl.createProgram()!;
    gl.attachShader(nextProgram, compile(gl.VERTEX_SHADER, vertex));
    gl.attachShader(nextProgram, compile(gl.FRAGMENT_SHADER, fragment));
    gl.linkProgram(nextProgram);
    if (!gl.getProgramParameter(nextProgram, gl.LINK_STATUS)) {
      throw Error(gl.getProgramInfoLog(nextProgram) || "Link error");
    }
    if (program) gl.deleteProgram(program);
    program = nextProgram;
    nextProgram = null;
    shaderSource = source;
    failedShader = "";
    shaderVertexArray ||= gl.createVertexArray();
    if (shaderVertexArray) gl.bindVertexArray(shaderVertexArray);
    for (const name of [
      "screen",
      "motion",
      "detail",
      "color0",
      "color1",
      "color2",
      "finish",
    ]) {
      locations[name] = gl.getUniformLocation(program, name);
    }
    backgroundError.hidden = true;
    if (!shaderLoopStarted) {
      shaderLoopStarted = true;
      requestAnimationFrame(drawShader);
    }
  } catch (error) {
    backgroundError.textContent =
      "Fundo dinâmico: " +
      (error instanceof Error ? error.message : String(error));
    backgroundError.hidden = false;
    failedShader = source;
    if (!program) canvas.style.display = "none";
  } finally {
    if (gl) {
      shaders.forEach((shader) => gl!.deleteShader(shader));
      if (nextProgram) gl.deleteProgram(nextProgram);
    }
    shaderStarting = false;
  }
}

function drawShader(now: number): void {
  if (gl && program && shaderState) {
    try {
      const s = shaderState;
      const p = options(s).shader || {};
      gl.viewport(0, 0, canvas.width, canvas.height);
      gl.useProgram(program);
      gl.uniform4f(
        locations.screen!,
        canvas.width,
        canvas.height,
        (s.clock || 0) + (now - shaderSync) / 1000,
        s.shaderIndex || 0,
      );
      gl.uniform4f(
        locations.motion!,
        p.speed ?? 0.2,
        p.scale ?? 1,
        p.intensity ?? 1,
        p.softness ?? 0.5,
      );
      gl.uniform4f(
        locations.detail!,
        p.detail ?? 4,
        ((p.direction ?? 0) * Math.PI) / 180,
        p.seed ?? 0,
        p.vignette ?? 0.4,
      );
      const colors =
        Array.isArray(p.colors) && p.colors.length >= 3
          ? p.colors
          : [
              [6, 12, 28],
              [24, 160, 130],
              [90, 60, 180],
            ];
      colors.forEach((col, i) =>
        gl!.uniform4f(
          locations["color" + i]!,
          col[0] / 255,
          col[1] / 255,
          col[2] / 255,
          1,
        ),
      );
      gl.uniform4f(locations.finish!, p.grain ?? 0.01, 0, 0, 0);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
    } catch {
      /* ignore */
    }
  }
  requestAnimationFrame(drawShader);
}

let polling = false;
let refreshPending = false;
let pollTimer: ReturnType<typeof setTimeout> | undefined;

async function poll(): Promise<void> {
  if (polling) {
    refreshPending = true;
    return;
  }
  polling = true;
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 2500);
  try {
    const response = await fetch(
      "/api/state/" + encodeURIComponent(screenId),
      { cache: "no-store", signal: controller.signal },
    );
    if (!response.ok) throw Error(String(response.status));
    const text = await response.text();
    failures = 0;
    connection.hidden = true;
    if (text !== last) {
      try {
        render(JSON.parse(text));
        last = text;
      } catch (err) {
        console.error("Erro ao renderizar tela:", err);
      }
    }
  } catch {
    if (++failures >= 2) {
      connection.textContent =
        "Conexão interrompida · reconectando…";
      connection.hidden = false;
      box.style.display = "none";
      title.textContent = "";
      last = "";
    }
  } finally {
    clearTimeout(timeout);
    polling = false;
    clearTimeout(pollTimer);
    const delay = refreshPending
      ? 0
      : screenId.startsWith("__control_")
        ? 100
        : 200;
    refreshPending = false;
    pollTimer = setTimeout(poll, delay);
  }
}

addEventListener("message", (event: MessageEvent) => {
  if (
    screenId.startsWith("__control_") &&
    event.origin === location.origin &&
    event.source === parent &&
    (event.data as { type?: string })?.type === "control-refresh"
  ) {
    clearTimeout(pollTimer);
    poll();
  }
});

addEventListener("resize", fit);
poll();
