'use strict';
const byId = id => document.getElementById(id);
const stage = byId('stage'), box = byId('box'), lyrics = byId('lyrics');
const background = byId('background'), video = byId('video'), canvas = byId('shader');
const layer = byId('current-layer'), layers = byId('text-layers');
const title = byId('title'), connection = byId('connection'), backgroundError = byId('background-error');
const stageClock = byId('stage-clock'), nextPreview = byId('next-preview');
const designLayer = document.createElement('canvas');
designLayer.id = 'design-layers';
designLayer.hidden = true;
Object.assign(designLayer.style, { position: 'absolute', inset: '0', width: '100%', height: '100%', pointerEvents: 'none', zIndex: '3' });
stage.append(designLayer);
let designRenderer = null;
import('/design.js').then(({ DesignRenderer }) => {
  designRenderer = new DesignRenderer(designLayer);
  if (current) { designRenderer.setScene(current.design); fit(); }
}).catch(error => console.error('Não foi possível carregar o design:', error));
const screenId = location.pathname.split('/').filter(Boolean).pop();
let last = '', current = null, failures = 0, fontUrl = '', pendingFont = 0;
let shaderState = null, shaderSync = 0, gl = null, program = null, locations = {};
let activeAnimations = [], shaderStarting = false;
let shaderSource = '', failedShader = '', shaderLoopStarted = false;
let shaderVertexArray = null;

function updateClock() {
  if (stageClock && !stageClock.hidden) {
    const d = new Date();
    stageClock.textContent = d.getHours().toString().padStart(2, '0') + ':' + d.getMinutes().toString().padStart(2, '0');
  }
}
setInterval(updateClock, 1000);

byId('fullscreen').onclick = async () => {
  try { if (document.fullscreenElement) await document.exitFullscreen(); else await document.documentElement.requestFullscreen(); }
  catch { connection.textContent = 'Use o modo de tela cheia do navegador (F11).'; connection.hidden = false; }
};

function options(s) {
  return { auto_fit: true, wrap_text: true, text_opacity: 1, text_glow: 0, paragraph_spacing: 0,
    text_rotation: 0, transition_ms: 400, transition_strength: 12, transition_easing: 'smooth',
    background_overlay: 0, background_brightness: 1, video: {}, shader: {}, ...s.options };
}

function scalePixels(css, scale) {
  return css.replace(/(-?[\d.]+)px/g, (_, n) => Number(n) * scale + 'px');
}

function fit() {
  if (!current) return;
  const s = current, o = options(s);
  let w = innerWidth, h = innerHeight;
  if (s.aspect > 0) { w = Math.min(w, h * s.aspect); h = w / s.aspect; }
  stage.style.width = w + 'px'; stage.style.height = h + 'px';
  const scale = h / 720, b = s.box;
  Object.assign(box.style, { left: b[0] * 100 + '%', top: b[1] * 100 + '%', width: b[2] * 100 + '%', height: b[3] * 100 + '%',
    alignItems: ({ top: 'flex-start', bottom: 'flex-end' })[s.vertical] || 'center',
    background: s.boxBackground, border: scalePixels(s.boxBorder || 'none', scale), borderRadius: s.radius * scale + 'px', padding: s.padding * scale + 'px' });
  lyrics.style.letterSpacing = s.letterSpacing * scale + 'px';
  const shadows = s.shadow && s.shadow !== 'none' ? [scalePixels(s.shadow, scale)] : [];
  if (o.text_glow > 0) shadows.push(`0 0 ${o.text_glow * scale}px ${s.color}`);
  lyrics.style.textShadow = shadows.join(',') || 'none';
  lyrics.style.webkitTextStroke = scalePixels(s.outline || '0 transparent', scale);
  lyrics.querySelectorAll('.paragraph').forEach((p, i, all) => p.style.marginBottom = i + 1 < all.length ? o.paragraph_spacing * scale + 'px' : '0');
  let size = Math.max(12, s.size * scale);
  const availableHeight = box.clientHeight - 2 * s.padding * scale;
  const availableWidth = box.clientWidth - 2 * s.padding * scale;
  const measure = n => { lyrics.style.fontSize = n + 'px'; return lyrics.scrollHeight <= availableHeight + 1 && lyrics.scrollWidth <= availableWidth + 1; };
  if (availableHeight > 20 && availableWidth > 20) {
    if (!measure(size) && o.auto_fit) {
      let low = 12, high = size;
      for (let i = 0; i < 18; i++) { const mid = (low + high) / 2; if (measure(mid)) low = mid; else high = mid; }
      size = Math.max(12, low);
    }
  }
  lyrics.style.fontSize = size + 'px';
  const resolutionScale = Math.min(devicePixelRatio || 1, 1.5, 1920 / w, 1080 / h);
  canvas.width = Math.max(1, Math.round(w * resolutionScale)); canvas.height = Math.max(1, Math.round(h * resolutionScale));
  designRenderer?.resize(w, h);
}

function transition(oldLayer, s) {
  const o = options(s), effect = s.transition || 'fade';
  activeAnimations.forEach(a => a.cancel()); activeAnimations = [];
  if (!oldLayer || !s.motion || effect === 'cut' || s.hidden || !s.text || matchMedia('(prefers-reduced-motion: reduce)').matches) { oldLayer?.remove(); return; }
  const scale = stage.clientHeight / 720, blur = o.transition_strength * scale;
  let incoming = [{ opacity: 0 }, { opacity: 1 }], outgoing = [{ opacity: 1 }, { opacity: 0 }];
  if (effect === 'blur') { incoming[0].filter = `blur(${blur}px)`; incoming[1].filter = 'blur(0px)'; outgoing[0].filter = 'blur(0px)'; outgoing[1].filter = `blur(${blur}px)`; }
  const axes = { slide_left: ['X', '100%', '-100%'], slide_right: ['X', '-100%', '100%'], slide_up: ['Y', '100%', '-100%'], slide_down: ['Y', '-100%', '100%'] };
  if (axes[effect]) { const [axis, start, end] = axes[effect]; incoming[0].transform = `translate${axis}(${start})`; incoming[1].transform = `translate${axis}(0)`; outgoing[0].transform = `translate${axis}(0)`; outgoing[1].transform = `translate${axis}(${end})`; }
  if (effect === 'zoom' || effect === 'cube') { incoming[0].transform = 'scale(.84)'; incoming[1].transform = 'scale(1)'; outgoing[0].transform = 'scale(1)'; outgoing[1].transform = 'scale(1.16)'; }
  if (effect === 'wipe') { incoming[0].clipPath = 'inset(0 100% 0 0)'; incoming[1].clipPath = 'inset(0 0 0 0)'; outgoing[0].clipPath = 'inset(0 0 0 0)'; outgoing[1].clipPath = 'inset(0 0 0 100%)'; }
  const timing = { duration: o.transition_ms, easing: ({ linear: 'linear', ease_out: 'cubic-bezier(.2,.8,.2,1)' })[o.transition_easing] || 'ease-in-out' };
  activeAnimations.push(layer.animate(incoming, timing));
  const oldAnimation = oldLayer.animate(outgoing, timing); activeAnimations.push(oldAnimation);
  oldAnimation.finished.then(() => oldLayer.remove()).catch(() => oldLayer.remove());
}

async function loadFont(s) {
  if (!s.fontUrl || s.fontUrl === fontUrl) return;
  fontUrl = s.fontUrl; const ticket = ++pendingFont;
  try {
    const face = await new FontFace('omni-live-' + ticket, `url(${JSON.stringify(s.fontUrl)})`, { weight: String(s.weight), style: s.italic ? 'italic' : 'normal' }).load();
    if (ticket !== pendingFont) return;
    document.fonts.add(face); lyrics.style.fontFamily = JSON.stringify(face.family) + ',Arial,sans-serif'; fit();
  } catch { lyrics.style.fontFamily = JSON.stringify(s.font) + ',Arial,sans-serif'; }
}

function appendFormattedRuns(parent, text) {
  // Parses simple markdown tokens: **bold**, *italic*, __underline__, ~~strike~~
  const regex = /(\*\*.*?\*\*|\*[^*]+?\*|__.*?__|~~.*?~~)/g;
  let lastIndex = 0;
  let match;
  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parent.append(document.createTextNode(text.slice(lastIndex, match.index)));
    }
    const token = match[0];
    let el;
    if (token.startsWith('**') && token.endsWith('**')) {
      el = document.createElement('strong');
      el.textContent = token.slice(2, -2);
    } else if (token.startsWith('*') && token.endsWith('*')) {
      el = document.createElement('em');
      el.textContent = token.slice(1, -1);
    } else if (token.startsWith('__') && token.endsWith('__')) {
      el = document.createElement('u');
      el.textContent = token.slice(2, -2);
    } else if (token.startsWith('~~') && token.endsWith('~~')) {
      el = document.createElement('s');
      el.textContent = token.slice(2, -2);
    } else {
      el = document.createTextNode(token);
    }
    parent.append(el);
    lastIndex = regex.lastIndex;
  }
  if (lastIndex < text.length) {
    parent.append(document.createTextNode(text.slice(lastIndex)));
  }
}

function render(s) {
  const o = options(s), changed = !current || current.text !== s.text;
  let oldLayer = null;
  if (changed) {
    layers.querySelectorAll('.previous-layer').forEach(el => el.remove());
    if (current?.text && !s.hidden) { oldLayer = layer.cloneNode(true); oldLayer.removeAttribute('id'); oldLayer.classList.add('previous-layer'); oldLayer.querySelectorAll('[id]').forEach(el => el.removeAttribute('id')); layers.prepend(oldLayer); }
    // Only text nodes and safe spans are created. Imported lyrics are never interpreted as arbitrary HTML.
    lyrics.replaceChildren();
    s.text.split('\n').forEach((text, index, all) => {
      if (text.trim().startsWith('#*')) {
        return;
      }
      const p = document.createElement('span');
      p.className = 'paragraph';
      if (text.trim().startsWith('//')) {
        p.classList.add('comment');
        p.textContent = text;
      } else if (text.includes('//')) {
        const commentIdx = text.indexOf('//');
        const mainPart = text.slice(0, commentIdx);
        const commentPart = text.slice(commentIdx);
        appendFormattedRuns(p, mainPart);
        const cSpan = document.createElement('span');
        cSpan.className = 'paragraph comment';
        cSpan.textContent = ' ' + commentPart.trim();
        p.append(cSpan);
      } else {
        appendFormattedRuns(p, text || '\u200b');
      }
      lyrics.append(p);
      if (index + 1 < all.length) lyrics.append(document.createTextNode('\n'));
    });
  }
  current = s;
  designLayer.hidden = s.hidden || !s.design;
  designRenderer?.setScene(s.design);
  document.title = s.name + ' · Omni Presenter';
  document.body.style.background = s.transparent ? 'transparent' : '#000';
  background.style.background = s.hidden ? (s.transparent ? 'transparent' : '#000') : s.background;
  box.style.display = s.hidden || !s.text ? 'none' : 'flex';
  title.textContent = s.title;
  if (stageClock) {
    stageClock.hidden = !s.showClock;
    if (s.showClock) updateClock();
  }
  if (nextPreview) {
    if (s.showNextPreview && s.nextText && s.nextText.trim()) {
      const firstLine = s.nextText.trim().split('\n')[0];
      nextPreview.textContent = '';
      const strong = document.createElement('strong');
      strong.textContent = 'A seguir:';
      nextPreview.append(strong, ' ' + firstLine);
      nextPreview.hidden = false;
    } else {
      nextPreview.hidden = true;
    }
  }
  lyrics.style.whiteSpace = 'normal';
  lyrics.querySelectorAll('.paragraph').forEach(p => p.style.whiteSpace = o.wrap_text ? 'pre-wrap' : 'pre');
  lyrics.style.opacity = o.text_opacity;
  lyrics.style.lineHeight = s.lineHeight;
  lyrics.style.color = s.color; lyrics.style.fontWeight = s.weight;
  lyrics.style.fontStyle = s.italic ? 'italic' : 'normal';
  lyrics.style.textDecoration = [s.underline ? 'underline' : '', s.strike ? 'line-through' : ''].join(' ');
  lyrics.style.textAlign = s.align;
  lyrics.style.textTransform = ({ uppercase: 'uppercase', lowercase: 'lowercase', titlecase: 'capitalize' })[s.case] || 'none';
  lyrics.style.transform = `rotate(${o.text_rotation}deg)`;
  lyrics.style.transformOrigin = ({ left: '0', right: '100%' })[s.align] || '50%';
  loadFont(s);
  fit();
  requestAnimationFrame(fit);
  const showBackground = !s.transparent && !s.hidden;
  const showVideo = showBackground && s.backgroundType === 'video' && s.videoUrl;
  video.style.display = showVideo ? 'block' : 'none';
  if (showVideo) {
    if (video.dataset.source !== s.videoUrl) { video.dataset.source = s.videoUrl; video.src = s.videoUrl; video.onloadedmetadata = () => { video.currentTime = Math.min(o.video.start || 0, video.duration || 0); video.play().catch(() => {}); }; }
    video.loop = o.video.repeat !== false; video.playbackRate = o.video.speed || 1;
    video.style.objectFit = o.video.fit || 'cover'; video.style.transform = o.video.mirror ? 'scaleX(-1)' : 'none';
    video.style.filter = `saturate(${o.video.saturation ?? 1}) contrast(${o.video.contrast ?? 1})`;
    if (video.paused && !video.ended) video.play().catch(() => {});
  } else video.pause();
  const brightness = o.background_brightness;
  byId('brightness').style.background = !showBackground ? 'transparent' : brightness < 1 ? `rgba(0,0,0,${1 - brightness})` : `rgba(255,255,255,${Math.min(1, (brightness - 1) * .35)})`;
  byId('dimmer').style.background = showBackground ? `rgba(0,0,0,${o.background_overlay})` : 'transparent';
  if (showBackground && s.backgroundType === 'shader') {
    shaderState = s; shaderSync = performance.now(); canvas.style.display = 'block';
    const source = s.shaderFragment || 'builtin';
    if ((!gl || shaderSource !== source) && failedShader !== source && !shaderStarting) startShader(source);
  }
  else { shaderState = null; canvas.style.display = 'none'; }
  if (changed) transition(oldLayer, s);
  if (s.hidden) { activeAnimations.forEach(a => a.cancel()); layers.querySelectorAll('.previous-layer').forEach(el => el.remove()); }
}

async function startShader(source) {
  shaderStarting = true;
  const shaders = [];
  let nextProgram = null;
  try {
    gl ||= canvas.getContext('webgl2', { alpha: false, antialias: false });
    if (!gl) throw Error('WebGL 2 indisponível neste navegador.');
    const compile = (type, source) => { const shader = gl.createShader(type); shaders.push(shader); gl.shaderSource(shader, source); gl.compileShader(shader); if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) throw Error(gl.getShaderInfoLog(shader)); return shader; };
    const fragment = source === 'builtin' ? await (await fetch('/shader.glsl')).text() : source;
    const vertex = '#version 300 es\nprecision highp float;out vec2 uv;void main(){vec2 p=vec2(float((gl_VertexID<<1)&2),float(gl_VertexID&2));gl_Position=vec4(p*vec2(2,-2)+vec2(-1,1),0,1);uv=p;}';
    nextProgram = gl.createProgram(); gl.attachShader(nextProgram, compile(gl.VERTEX_SHADER, vertex)); gl.attachShader(nextProgram, compile(gl.FRAGMENT_SHADER, fragment)); gl.linkProgram(nextProgram);
    if (!gl.getProgramParameter(nextProgram, gl.LINK_STATUS)) throw Error(gl.getProgramInfoLog(nextProgram));
    if (program) gl.deleteProgram(program);
    program = nextProgram; nextProgram = null; shaderSource = source; failedShader = '';
    shaderVertexArray ||= gl.createVertexArray();
    gl.bindVertexArray(shaderVertexArray);
    for (const name of ['screen', 'motion', 'detail', 'color0', 'color1', 'color2', 'finish']) locations[name] = gl.getUniformLocation(program, name);
    backgroundError.hidden = true;
    if (!shaderLoopStarted) { shaderLoopStarted = true; requestAnimationFrame(drawShader); }
  } catch (error) { backgroundError.textContent = 'Fundo dinâmico: ' + error.message; backgroundError.hidden = false; failedShader = source; if (!program) canvas.style.display = 'none'; }
  finally { if (gl) { shaders.forEach(shader => gl.deleteShader(shader)); if (nextProgram) gl.deleteProgram(nextProgram); } shaderStarting = false; }
}

function drawShader(now) {
  if (gl && program && shaderState) {
    const s = shaderState, p = options(s).shader;
    gl.viewport(0, 0, canvas.width, canvas.height); gl.useProgram(program);
    gl.uniform4f(locations.screen, canvas.width, canvas.height, s.clock + (now - shaderSync) / 1000, s.shaderIndex);
    gl.uniform4f(locations.motion, p.speed, p.scale, p.intensity, p.softness);
    gl.uniform4f(locations.detail, p.detail, p.direction * Math.PI / 180, p.seed, p.vignette);
    p.colors.forEach((color, i) => gl.uniform4f(locations['color' + i], ...color.map(n => n / 255), 1));
    gl.uniform4f(locations.finish, p.grain, 0, 0, 0); gl.drawArrays(gl.TRIANGLES, 0, 3);
  }
  requestAnimationFrame(drawShader);
}

let polling = false, refreshPending = false, pollTimer;
async function poll() {
  if (polling) { refreshPending = true; return; }
  polling = true;
  const controller = new AbortController(), timeout = setTimeout(() => controller.abort(), 2500);
  try {
    const response = await fetch('/api/state/' + encodeURIComponent(screenId), { cache: 'no-store', signal: controller.signal });
    if (!response.ok) throw Error(response.status);
    const text = await response.text();
    if (text !== last) { render(JSON.parse(text)); last = text; }
    failures = 0; connection.hidden = true;
  } catch {
    if (++failures >= 2) { connection.textContent = 'Conexão interrompida · reconectando…'; connection.hidden = false; box.style.display = 'none'; title.textContent = ''; last = ''; }
  } finally {
    clearTimeout(timeout); polling = false; clearTimeout(pollTimer);
    const delay = refreshPending ? 0 : screenId.startsWith('__control_') ? 100 : 200;
    refreshPending = false; pollTimer = setTimeout(poll, delay);
  }
}
addEventListener('message', event => {
  if (screenId.startsWith('__control_') && event.origin === location.origin && event.source === parent && event.data?.type === 'control-refresh') {
    clearTimeout(pollTimer); poll();
  }
});
addEventListener('resize', fit);
poll();
