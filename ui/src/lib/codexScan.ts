// Reads a screenshot of the game's Blueprints page, on this PC only. Learned blueprints are
// the blue tiles, so it finds those and matches each tile's picture against the known icons.

import { iconSrc } from "./api";

export interface ScanEntry {
  id: string;
  name: string;
  icon: string | null;
}

export interface ScanResult {
  /** Best few candidates with their scores, for checking the matcher. */
  top: { id: string; score: number }[];
  entry: ScanEntry | null;
  /** Second-best guess, for telling look-alikes apart when reviewing. */
  other: ScanEntry | null;
  /** 0 to 1. Close icons, like the Mk. 3 variants, score low even when right. */
  confidence: number;
  crop: string;
}

const WORK_WIDTH = 2000;
const STEP = 2;
const SIZE = 40;
const INSET = 4;
const COLOUR_WEIGHT = 1.5;
/** Trims tried per tile: light keeps thin items whole, heavy ignores wires and antennas. */
const TRIMS = [0.005, 0.02];
const CONFIDENT = 0.12;

interface Box {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

function loadImage(src: string, cross: boolean) {
  return new Promise<HTMLImageElement>((resolve, reject) => {
    const image = new Image();
    if (cross) image.crossOrigin = "anonymous";
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error(`Couldn't load ${src.slice(0, 60)}`));
    image.src = src;
  });
}

function canvas(width: number, height: number) {
  const c = document.createElement("canvas");
  c.width = width;
  c.height = height;
  return { c, ctx: c.getContext("2d", { willReadFrequently: true })! };
}

const isBlue = (r: number, g: number, b: number) => b > 45 && b > r * 1.5 && b > g * 1.25;
/** Tile background, including its darker corners. An icon's steel is close to neutral. */
const isBackdrop = (r: number, g: number, b: number) => b > r * 1.3 && b > g * 1.12;

/** Connected blue regions, at half size to keep it quick. */
function blueRegions(data: Uint8ClampedArray, width: number, height: number): Box[] {
  const [w, h] = [Math.floor(width / STEP), Math.floor(height / STEP)];
  const mask = new Uint8Array(w * h);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const i = (y * STEP * width + x * STEP) * 4;
      if (isBlue(data[i], data[i + 1], data[i + 2])) mask[y * w + x] = 1;
    }
  }
  const boxes: Box[] = [];
  const stack: number[] = [];
  for (let start = 0; start < mask.length; start++) {
    if (!mask[start]) continue;
    const box = { x0: w, y0: h, x1: 0, y1: 0 };
    stack.push(start);
    mask[start] = 0;
    while (stack.length) {
      const at = stack.pop()!;
      const [x, y] = [at % w, Math.floor(at / w)];
      box.x0 = Math.min(box.x0, x);
      box.y0 = Math.min(box.y0, y);
      box.x1 = Math.max(box.x1, x);
      box.y1 = Math.max(box.y1, y);
      for (const next of [at - 1, at + 1, at - w, at + w]) {
        if (next >= 0 && next < mask.length && mask[next]) {
          mask[next] = 0;
          stack.push(next);
        }
      }
    }
    boxes.push({ x0: box.x0 * STEP, y0: box.y0 * STEP, x1: box.x1 * STEP, y1: box.y1 * STEP });
  }
  return boxes;
}

const median = (values: number[]) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];

/** A gun drawn across a tile can split its blue in two, so snap everything to the grid. */
function tilesFrom(regions: Box[]): Box[] {
  const big = regions.filter((b) => b.x1 - b.x0 > 30 && b.y1 - b.y0 > 24);
  if (!big.length) return [];
  const tileW = median(big.map((b) => b.x1 - b.x0));
  const tileH = median(big.map((b) => b.y1 - b.y0));
  const whole = big.filter((b) => Math.abs(b.x1 - b.x0 - tileW) < tileW * 0.12 && Math.abs(b.y1 - b.y0 - tileH) < tileH * 0.12);
  const lefts = [...new Set(whole.map((b) => b.x0))].sort((a, b) => a - b);
  const tops = [...new Set(whole.map((b) => b.y0))].sort((a, b) => a - b);
  const gaps = (list: number[], size: number) => list.slice(1).map((v, i) => v - list[i]).filter((g) => g > size * 0.9 && g < size * 1.5);
  const pitchX = gaps(lefts, tileW).length ? median(gaps(lefts, tileW)) : tileW * 1.08;
  const pitchY = gaps(tops, tileH).length ? median(gaps(tops, tileH)) : tileH * 1.33;
  const [x0, y0] = [Math.min(...whole.map((b) => b.x0)), Math.min(...whole.map((b) => b.y0))];
  const colOf = (x: number) => Math.round((x - x0) / pitchX);
  const rowOf = (y: number) => Math.round((y - y0) / pitchY);
  // Measured edges per line: a small pitch error would otherwise grow across the grid.
  const edges = (values: [number, number][]) => {
    const groups = new Map<number, number[]>();
    for (const [line, value] of values) groups.set(line, [...(groups.get(line) ?? []), value]);
    return new Map([...groups].map(([line, list]) => [line, median(list)]));
  };
  const columnX = edges(whole.map((b) => [colOf(b.x0), b.x0]));
  const rowY = edges(whole.map((b) => [rowOf(b.y0), b.y0]));
  const cells = new Map<string, Box>();
  for (const b of big.filter((r) => r.x1 - r.x0 < tileW * 1.2 && r.y1 - r.y0 < tileH * 1.2)) {
    const col = colOf((b.x0 + b.x1) / 2 - tileW / 2);
    const row = rowOf((b.y0 + b.y1) / 2 - tileH / 2);
    const [x, y] = [columnX.get(col) ?? x0 + col * pitchX, rowY.get(row) ?? y0 + row * pitchY];
    cells.set(`${row}:${col}`, { x0: x, y0: y, x1: x + tileW, y1: y + tileH });
  }
  return [...cells.values()].sort((a, b) => a.y0 - b.y0 || a.x0 - b.x0);
}

interface Print {
  gray: Float32Array;
  /** Colour away from grey, as red minus green and blue minus green. */
  tint: Float32Array;
  mask: Uint8Array;
}

/**
 * The part of an image that is the object, as a box. The edges trim off the outermost few
 * percent of it, so thin wires and antennas don't stretch the box. The tick in a tile's
 * corner doesn't count.
 */
function objectBox(data: Uint8ClampedArray, width: number, height: number, inTile: boolean, trim: number): Box | null {
  const cols = new Float64Array(width);
  const rows = new Float64Array(height);
  let total = 0;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      const solid = inTile
        ? !isBackdrop(data[i], data[i + 1], data[i + 2]) && !(x > width * 0.72 && y < height * 0.3)
        : data[i + 3] > 128;
      if (!solid) continue;
      cols[x]++;
      rows[y]++;
      total++;
    }
  }
  if (!total) return null;
  const span = (counts: Float64Array) => {
    let [seen, low, high] = [0, 0, counts.length - 1];
    for (let i = 0; i < counts.length; i++) {
      seen += counts[i];
      if (seen < total * trim) low = i + 1;
    }
    seen = 0;
    for (let i = counts.length - 1; i >= 0; i--) {
      seen += counts[i];
      if (seen < total * trim) high = i - 1;
    }
    return [low, Math.max(low, high)];
  };
  const [x0, x1] = span(cols);
  const [y0, y1] = span(rows);
  return { x0, y0, x1, y1 };
}

/** Scales the object to fill a square, so size and position in the source stop mattering. */
function print(source: CanvasImageSource, box: Box, inTile: boolean): Print {
  const { ctx } = canvas(SIZE, SIZE);
  const [w, h] = [box.x1 - box.x0 + 1, box.y1 - box.y0 + 1];
  const scale = SIZE / Math.max(w, h);
  const [dw, dh] = [w * scale, h * scale];
  ctx.drawImage(source, box.x0, box.y0, w, h, (SIZE - dw) / 2, (SIZE - dh) / 2, dw, dh);
  const data = ctx.getImageData(0, 0, SIZE, SIZE).data;
  const gray = new Float32Array(SIZE * SIZE);
  const tint = new Float32Array(SIZE * SIZE * 2);
  const mask = new Uint8Array(SIZE * SIZE);
  for (let i = 0; i < SIZE * SIZE; i++) {
    const [r, g, b, alpha] = [data[i * 4], data[i * 4 + 1], data[i * 4 + 2], data[i * 4 + 3]];
    gray[i] = 0.3 * r + 0.59 * g + 0.11 * b;
    tint[i * 2] = r - g;
    tint[i * 2 + 1] = b - g;
    mask[i] = alpha > 128 && (!inTile || !isBackdrop(r, g, b)) ? 1 : 0;
  }
  return { gray, tint, mask };
}

/** Shape overlap plus how well light and dark line up where the shapes meet. */
function similarity(tile: Print, icon: Print) {
  let [both, either, sumA, sumB] = [0, 0, 0, 0];
  for (let i = 0; i < tile.mask.length; i++) {
    if (tile.mask[i] || icon.mask[i]) either++;
    if (tile.mask[i] && icon.mask[i]) {
      both++;
      sumA += tile.gray[i];
      sumB += icon.gray[i];
    }
  }
  if (both < 8) return 0;
  const [meanA, meanB] = [sumA / both, sumB / both];
  let [cross, varA, varB, colour] = [0, 0, 0, 0];
  for (let i = 0; i < tile.mask.length; i++) {
    if (!tile.mask[i] || !icon.mask[i]) continue;
    const [da, db] = [tile.gray[i] - meanA, icon.gray[i] - meanB];
    cross += da * db;
    colour += Math.abs(tile.tint[i * 2] - icon.tint[i * 2]) + Math.abs(tile.tint[i * 2 + 1] - icon.tint[i * 2 + 1]);
    varA += da * da;
    varB += db * db;
  }
  const texture = cross / (Math.sqrt(varA * varB) || 1);
  // Red and yellow light sticks share a shape; only colour tells them apart.
  return both / either + 0.6 * texture - COLOUR_WEIGHT * (colour / both / 255);
}

/** One print per trim, for the source's object, or null when there's nothing in it. */
function prints(source: HTMLCanvasElement, inTile: boolean): Print[] | null {
  const data = source.getContext("2d")!.getImageData(0, 0, source.width, source.height).data;
  const boxes = TRIMS.map((trim) => objectBox(data, source.width, source.height, inTile, trim));
  return boxes.every(Boolean) ? boxes.map((box) => print(source, box!, inTile)) : null;
}

async function iconPrints(entry: ScanEntry) {
  if (!entry.icon) return null;
  const image = await loadImage(iconSrc(entry.icon), true);
  const { c, ctx } = canvas(image.naturalWidth, image.naturalHeight);
  ctx.drawImage(image, 0, 0);
  return prints(c, false);
}

export async function scanCodex(file: Blob, entries: ScanEntry[], progress: (done: number, total: number) => void = () => {}): Promise<ScanResult[]> {
  // A data URL, because the app's content policy doesn't allow blob images.
  const url = await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
  {
    const shot = await loadImage(url, false);
    const height = Math.round((shot.naturalHeight / shot.naturalWidth) * WORK_WIDTH);
    const { c, ctx } = canvas(WORK_WIDTH, height);
    ctx.drawImage(shot, 0, 0, WORK_WIDTH, height);
    const tiles = tilesFrom(blueRegions(ctx.getImageData(0, 0, WORK_WIDTH, height).data, WORK_WIDTH, height));
    const known = (await Promise.all(entries.map(async (entry) => {
      try {
        return { entry, prints: await iconPrints(entry) };
      } catch {
        return { entry, prints: null };
      }
    }))).filter((k): k is { entry: ScanEntry; prints: Print[] } => !!k.prints);

    const scored = tiles.map((tile, index) => {
      const [w, h] = [tile.x1 - tile.x0, tile.y1 - tile.y0];
      const side = Math.min(w, h);
      // Inset, so the tile's border isn't taken for part of the object.
      const piece = canvas(Math.round(w) - 2 * INSET, Math.round(h) - 2 * INSET);
      piece.ctx.drawImage(c, tile.x0 + INSET, tile.y0 + INSET, piece.c.width, piece.c.height, 0, 0, piece.c.width, piece.c.height);
      const whole = { x0: 0, y0: 0, x1: piece.c.width - 1, y1: piece.c.height - 1 };
      const seen = prints(piece.c, true) ?? TRIMS.map(() => print(piece.c, whole, true));
      const crop = canvas(96, 96);
      crop.ctx.drawImage(c, tile.x0 + (w - side) / 2, tile.y0 + (h - side) / 2, side, side, 0, 0, 96, 96);
      const ranked = known
        .map((k) => ({ entry: k.entry, score: Math.max(...seen.map((p, t) => similarity(p, k.prints[t]))) }))
        .sort((a, b) => b.score - a.score);
      progress(index + 1, tiles.length);
      return { ranked, crop: crop.c.toDataURL("image/jpeg", 0.8) };
    });

    // Each blueprint exists once, so the surest tiles claim their match first.
    const taken = new Set<string>();
    const order = [...scored.keys()].sort((a, b) => scored[b].ranked[0].score - scored[a].ranked[0].score);
    const results: ScanResult[] = new Array(scored.length);
    for (const i of order) {
      const free = scored[i].ranked.filter((r) => !taken.has(r.entry.id));
      const [first, second] = free;
      if (first) taken.add(first.entry.id);
      results[i] = {
        top: scored[i].ranked.slice(0, 3).map((r) => ({ id: r.entry.id, score: Math.round(r.score * 1000) / 1000 })),
        entry: first?.entry ?? null,
        other: second?.entry ?? null,
        confidence: first && second ? Math.min(1, (first.score - second.score) / CONFIDENT) : 0,
        crop: scored[i].crop,
      };
    }
    return results;
  }
}
