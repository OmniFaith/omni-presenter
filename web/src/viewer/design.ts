export interface DesignLayerStyle {
  italic?: boolean;
}

export interface DesignShapeContent {
  radius: number;
}

export interface DesignImageContent {
  cover?: boolean;
}

export interface DesignLayerContent {
  Shape?: DesignShapeContent;
  Image?: DesignImageContent;
}

export interface DesignLayer {
  visible?: boolean;
  opacity: number;
  rect: [number, number, number, number];
  blend?: string;
  color?: [number, number, number, number];
  content?: DesignLayerContent;
  imageUrl?: string;
  fontUrl?: string;
  font_size?: number;
  font_weight?: number;
  bold?: boolean;
  text_style?: DesignLayerStyle;
  displayText?: string;
  align?: CanvasTextAlign;
}

export interface DesignScene {
  background_mode?: string;
  background?: [number, number, number, number];
  layers?: DesignLayer[];
}

export class DesignRenderer {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D | null;
  images = new Map<string, HTMLImageElement>();
  fonts = new Map<string, { name: string; ready: boolean }>();
  scene: DesignScene | null = null;
  fingerprint = "";
  width = 0;
  height = 0;
  queued = false;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
  }

  setScene(scene: DesignScene | null): void {
    const fingerprint = JSON.stringify(scene);
    if (fingerprint === this.fingerprint) return;
    this.fingerprint = fingerprint;
    this.scene = scene;
    for (const layer of scene?.layers || []) {
      if (layer.imageUrl && !this.images.has(layer.imageUrl)) {
        const image = new Image();
        image.onload = () => this.schedule();
        image.onerror = () => this.schedule();
        image.src = layer.imageUrl;
        this.images.set(layer.imageUrl, image);
      }
      if (layer.fontUrl && !this.fonts.has(layer.fontUrl)) {
        const name = "design-font-" + this.fonts.size;
        this.fonts.set(layer.fontUrl, { name, ready: false });
        const font = new FontFace(
          name,
          `url(${JSON.stringify(layer.fontUrl)})`,
          {
            weight: String(layer.font_weight || (layer.bold ? 700 : 400)),
            style: layer.text_style?.italic ? "italic" : "normal",
          },
        );
        font
          .load()
          .then((face) => {
            document.fonts.add(face);
            const cached = this.fonts.get(layer.fontUrl!);
            if (cached) cached.ready = true;
            this.schedule();
          })
          .catch(() => this.schedule());
      }
    }
    this.schedule();
  }

  resize(width: number, height: number): void {
    const ratio = Math.min(devicePixelRatio || 1, 1.5);
    const w = Math.max(1, Math.round(width * ratio));
    const h = Math.max(1, Math.round(height * ratio));
    if (this.canvas.width === w && this.canvas.height === h) return;
    this.canvas.width = w;
    this.canvas.height = h;
    this.width = width;
    this.height = height;
    this.schedule();
  }

  schedule(): void {
    if (this.queued) return;
    this.queued = true;
    requestAnimationFrame(() => {
      this.queued = false;
      this.draw();
    });
  }

  draw(): void {
    const ctx = this.ctx,
      w = this.width,
      h = this.height;
    if (!ctx || !w || !h) return;
    ctx.setTransform(this.canvas.width / w, 0, 0, this.canvas.height / h, 0, 0);
    ctx.clearRect(0, 0, w, h);
    if (!this.scene) return;
    if (this.scene.background_mode === "color") {
      ctx.fillStyle = color(this.scene.background);
      ctx.fillRect(0, 0, w, h);
    }
    for (const layer of this.scene.layers || []) {
      if (!layer.visible || layer.opacity <= 0) continue;
      const [rx, ry, rw, rh] = layer.rect;
      const x = rx * w,
        y = ry * h,
        width = rw * w,
        height = rh * h;
      ctx.save();
      ctx.globalAlpha = layer.opacity;
      ctx.globalCompositeOperation =
        ({
          Multiply: "multiply",
          Screen: "screen",
          Overlay: "overlay",
        } as Record<string, GlobalCompositeOperation>)[layer.blend || ""] ||
        "source-over";
      ctx.beginPath();
      ctx.rect(x, y, width, height);
      ctx.clip();
      const shape = layer.content?.Shape;
      if (shape) {
        ctx.fillStyle = color(layer.color);
        ctx.beginPath();
        ctx.roundRect(
          x,
          y,
          width,
          height,
          Math.max(0, Math.min((shape.radius * h) / 720, width / 2, height / 2)),
        );
        ctx.fill();
      } else if (layer.imageUrl) {
        const image = this.images.get(layer.imageUrl);
        if (image?.complete && image.naturalWidth) {
          const scale = layer.content?.Image?.cover
            ? Math.max(width / image.naturalWidth, height / image.naturalHeight)
            : Math.min(width / image.naturalWidth, height / image.naturalHeight);
          const iw = image.naturalWidth * scale,
            ih = image.naturalHeight * scale;
          ctx.drawImage(
            image,
            x + (width - iw) / 2,
            y + (height - ih) / 2,
            iw,
            ih,
          );
        }
      } else if (typeof layer.displayText === "string") {
        this.text(layer, x, y, width, height);
      }
      ctx.restore();
    }
  }

  text(
    layer: DesignLayer,
    x: number,
    y: number,
    width: number,
    height: number,
  ): void {
    const ctx = this.ctx;
    if (!ctx) return;
    const loaded = layer.fontUrl ? this.fonts.get(layer.fontUrl) : undefined;
    const family = loaded?.ready
      ? JSON.stringify(loaded.name)
      : "Arial, sans-serif";
    let size = Math.max(6, (layer.font_size || 40) * this.height / 720);
    let lines: string[] = [];
    const weight = layer.font_weight || (layer.bold ? 700 : 400);
    const setFont = () => {
      ctx.font = `${layer.text_style?.italic ? "italic " : ""}${weight} ${size}px ${family}`;
    };
    for (let pass = 0; pass < 32; pass++) {
      setFont();
      lines = wrap(ctx, layer.displayText || "", width);
      if (lines.length * size * 1.18 <= height || size <= 8) break;
      size *= 0.9;
    }
    ctx.fillStyle = color(layer.color);
    ctx.textAlign = layer.align || "center";
    ctx.textBaseline = "middle";
    const px =
      layer.align === "left"
        ? x
        : layer.align === "right"
          ? x + width
          : x + width / 2;
    const top = y + height / 2 - ((lines.length - 1) * size * 1.18) / 2;
    lines.forEach((line, index) =>
      ctx.fillText(line, px, top + index * size * 1.18),
    );
  }
}

function color(rgba: [number, number, number, number] = [255, 255, 255, 255]): string {
  return `rgba(${rgba[0]},${rgba[1]},${rgba[2]},${rgba[3] / 255})`;
}

function wrap(ctx: CanvasRenderingContext2D, text: string, width: number): string[] {
  const lines: string[] = [];
  for (const paragraph of text.split("\n")) {
    let line = "";
    for (const word of paragraph.split(/\s+/)) {
      const candidate = line ? line + " " + word : word;
      if (line && ctx.measureText(candidate).width > width) {
        lines.push(line);
        line = "";
      }
      if (ctx.measureText(word).width > width) {
        for (const character of word) {
          if (line && ctx.measureText(line + character).width > width) {
            lines.push(line);
            line = "";
          }
          line += character;
        }
      } else {
        line = line ? line + " " + word : word;
      }
    }
    lines.push(line);
  }
  return lines;
}

