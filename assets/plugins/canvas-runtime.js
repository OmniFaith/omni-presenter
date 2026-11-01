// Portable Canvas 2D recorder used by the native JavaScript plugin runtime.
// Browser outputs use their real Canvas 2D implementation with the same render(api, ctx) contract.
globalThis.OmniCanvas = class {
  constructor(width, height) {
    this.canvas = { width, height }; this.width = width; this.height = height; this.items = []; this.defs = []; this.path = []; this.stack = [];
    this.fillStyle = '#fff'; this.strokeStyle = '#fff'; this.lineWidth = 1; this.globalAlpha = 1; this.font = '32px sans-serif';
    this.textAlign = 'start'; this.textBaseline = 'alphabetic'; this.lineCap = 'butt'; this.lineJoin = 'miter'; this.matrix = [1, 0, 0, 1, 0, 0]; this.clipId = '';
  }
  escape(value) { return String(value).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;' }[c])); }
  paint(value) { if (value && value.stops) { const id = 'gradient' + this.defs.length; this.defs.push(`<linearGradient id="${id}" gradientUnits="userSpaceOnUse" x1="${value.x1}" y1="${value.y1}" x2="${value.x2}" y2="${value.y2}">${value.stops.map(s => `<stop offset="${s[0]}" stop-color="${this.escape(s[1])}"/>`).join('')}</linearGradient>`); return `url(#${id})`; } return this.escape(value); }
  style(stroke = false) { return `transform="matrix(${this.matrix.join(' ')})" opacity="${this.globalAlpha}" ${this.clipId ? `clip-path="url(#${this.clipId})"` : ''} ${stroke ? `fill="none" stroke="${this.paint(this.strokeStyle)}" stroke-width="${this.lineWidth}" stroke-linecap="${this.lineCap}" stroke-linejoin="${this.lineJoin}"` : `fill="${this.paint(this.fillStyle)}"`}`; }
  save() { const state = {}; for (const k of ['fillStyle', 'strokeStyle', 'lineWidth', 'globalAlpha', 'font', 'textAlign', 'textBaseline', 'lineCap', 'lineJoin', 'clipId']) state[k] = this[k]; state.matrix = this.matrix.slice(); this.stack.push(state); }
  restore() { if (this.stack.length) Object.assign(this, this.stack.pop()); }
  transform(a, b, c, d, e, f) { const m = this.matrix; this.matrix = [m[0] * a + m[2] * b, m[1] * a + m[3] * b, m[0] * c + m[2] * d, m[1] * c + m[3] * d, m[0] * e + m[2] * f + m[4], m[1] * e + m[3] * f + m[5]]; }
  setTransform(a, b, c, d, e, f) { this.matrix = typeof a === 'object' ? [a.a, a.b, a.c, a.d, a.e, a.f] : [a, b, c, d, e, f]; }
  resetTransform() { this.matrix = [1, 0, 0, 1, 0, 0]; }
  translate(x, y) { this.transform(1, 0, 0, 1, x, y); }
  scale(x, y) { this.transform(x, 0, 0, y, 0, 0); }
  rotate(a) { this.transform(Math.cos(a), Math.sin(a), -Math.sin(a), Math.cos(a), 0, 0); }
  beginPath() { this.path = []; }
  closePath() { this.path.push('Z'); }
  moveTo(x, y) { this.path.push(`M${x} ${y}`); }
  lineTo(x, y) { this.path.push(`L${x} ${y}`); }
  bezierCurveTo(a, b, c, d, e, f) { this.path.push(`C${a} ${b} ${c} ${d} ${e} ${f}`); }
  quadraticCurveTo(a, b, c, d) { this.path.push(`Q${a} ${b} ${c} ${d}`); }
  rect(x, y, w, h) { this.path.push(`M${x} ${y}h${w}v${h}h${-w}Z`); }
  roundRect(x, y, w, h, r = 0) { r = Array.isArray(r) ? r[0] : r; r = Math.max(0, Math.min(r, w / 2, h / 2)); this.path.push(`M${x + r} ${y}h${w - 2 * r}a${r} ${r} 0 0 1 ${r} ${r}v${h - 2 * r}a${r} ${r} 0 0 1 ${-r} ${r}h${2 * r - w}a${r} ${r} 0 0 1 ${-r} ${-r}v${2 * r - h}a${r} ${r} 0 0 1 ${r} ${-r}Z`); }
  arc(x, y, r, start, end, anticlockwise = false) { this.ellipse(x, y, r, r, 0, start, end, anticlockwise); }
  ellipse(x, y, rx, ry, rotation, start, end, anticlockwise = false) {
    let delta = end - start; const full = Math.abs(delta) >= Math.PI * 2;
    if (full) delta = anticlockwise ? -Math.PI * 2 : Math.PI * 2; else if (anticlockwise) { while (delta > 0) delta -= Math.PI * 2; } else { while (delta < 0) delta += Math.PI * 2; }
    const point = a => [x + Math.cos(rotation) * rx * Math.cos(a) - Math.sin(rotation) * ry * Math.sin(a), y + Math.sin(rotation) * rx * Math.cos(a) + Math.cos(rotation) * ry * Math.sin(a)];
    const first = point(start); this.path.push(`${this.path.length ? 'L' : 'M'}${first[0]} ${first[1]}`);
    const steps = full ? 2 : 1; for (let i = 1; i <= steps; i++) { const p = point(start + delta * i / steps); this.path.push(`A${rx} ${ry} ${rotation * 180 / Math.PI} ${Math.abs(delta / steps) > Math.PI ? 1 : 0} ${anticlockwise ? 0 : 1} ${p[0]} ${p[1]}`); }
  }
  fill(rule = 'nonzero') { this.items.push(`<path d="${this.path.join(' ')}" ${this.style()} fill-rule="${rule}"/>`); }
  stroke() { this.items.push(`<path d="${this.path.join(' ')}" ${this.style(true)}/>`); }
  clip() { const id = 'clip' + this.defs.length; this.defs.push(`<clipPath id="${id}"><path d="${this.path.join(' ')}"/></clipPath>`); this.clipId = id; }
  fillRect(x, y, w, h) { this.items.push(`<rect x="${x}" y="${y}" width="${w}" height="${h}" ${this.style()}/>`); }
  strokeRect(x, y, w, h) { this.items.push(`<rect x="${x}" y="${y}" width="${w}" height="${h}" ${this.style(true)}/>`); }
  clearRect(x, y, w, h) { if (x === 0 && y === 0 && w >= this.width && h >= this.height) this.items = []; }
  createLinearGradient(x1, y1, x2, y2) { return { x1, y1, x2, y2, stops: [], addColorStop(position, color) { this.stops.push([Math.max(0, Math.min(1, position)), color]); } }; }
  measureText(text) { const size = Number((this.font.match(/([\d.]+)px/) || [])[1] || 32); return { width: Array.from(String(text)).length * size * .6, actualBoundingBoxAscent: size * .8, actualBoundingBoxDescent: size * .2 }; }
  fillText(text, x, y) { this.text(text, x, y, false); }
  strokeText(text, x, y) { this.text(text, x, y, true); }
  text(text, x, y, stroke) { const match = this.font.match(/([\d.]+)px\s*(.*)/); const size = match ? match[1] : 32; const family = match ? match[2] : 'sans-serif'; this.items.push(`<text x="${x}" y="${y}" font-family="${this.escape(family)}" font-size="${size}" font-weight="${/bold|[7-9]00/.test(this.font) ? 700 : 400}" font-style="${/italic/.test(this.font) ? 'italic' : 'normal'}" text-anchor="${({ center: 'middle', right: 'end', end: 'end' })[this.textAlign] || 'start'}" dominant-baseline="${({ top: 'text-before-edge', hanging: 'hanging', middle: 'central', bottom: 'text-after-edge' })[this.textBaseline] || 'alphabetic'}" ${this.style(stroke)}>${this.escape(text)}</text>`); }
  toSVG() { return `<svg xmlns="http://www.w3.org/2000/svg" width="${this.width}" height="${this.height}" viewBox="0 0 ${this.width} ${this.height}"><defs>${this.defs.join('')}</defs>${this.items.join('')}</svg>`; }
};
globalThis.OmniPresenterCanvas = globalThis.OmniCanvas;
