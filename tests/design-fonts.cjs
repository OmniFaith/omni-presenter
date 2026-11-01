const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

(async () => {
  const source = fs.readFileSync(path.join(__dirname, '../assets/web/viewer/design.js'), 'utf8');
  const { DesignRenderer } = await import('data:text/javascript;base64,' + Buffer.from(source).toString('base64'));
  const faces = [];
  global.FontFace = class {
    constructor(name, url, options) { faces.push(options); }
    load() { return Promise.resolve(this); }
  };
  global.document = { fonts: { add() {} } };
  global.requestAnimationFrame = () => {};
  const ctx = {
    font: '',
    measureText: text => ({ width: text.length * 10 }),
    fillText() {},
  };
  const renderer = new DesignRenderer({ getContext: () => ctx });
  renderer.height = 720;
  for (const [weight, bold, italic, expected] of [
    [300, false, false, '300'],
    [800, false, true, 'italic 800'],
    [undefined, true, false, '700'],
  ]) {
    const layer = {
      fontUrl: '/font-' + expected,
      font_weight: weight,
      bold,
      text_style: { italic },
      font_size: 40,
      displayText: 'Texto',
    };
    renderer.setScene({ layers: [layer] });
    renderer.text(layer, 0, 0, 500, 200);
    assert.ok(ctx.font.startsWith(expected + ' 40px '), ctx.font);
    assert.equal(faces.at(-1).weight, String(weight || (bold ? 700 : 400)));
    assert.equal(faces.at(-1).style, italic ? 'italic' : 'normal');
  }
  console.log('Design fonts: numeric weights, italic and legacy bold passed.');
})().catch(error => { console.error(error); process.exitCode = 1; });
