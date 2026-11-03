// Map marker icons, drawn here from simple shapes so nothing of Embark's or Map Genie's ships.
// Each icon is a coloured badge for its group with a white glyph for its type, rendered once
// per size and reused, which keeps drawing tens of thousands of markers cheap.

export const GROUPS: Record<string, { label: string; color: string }> = {
  locations: { label: "Locations", color: "#e8b923" },
  quests: { label: "Quests", color: "#d45cf0" },
  containers: { label: "Loot", color: "#2a9fb8" },
  arc: { label: "ARC", color: "#d8404a" },
  events: { label: "Events", color: "#ea7a35" },
  nature: { label: "Nature", color: "#3aa85c" },
  route: { label: "Route stop", color: "#f5c400" },
};

// Glyphs on a 24-unit grid, stroked in white.
const GLYPHS: Record<string, string> = {
  extraction: "M12 4l6 6h-4v6h-4v-6H6z M5 20h14",
  hatch: "M5 6h14v12H5z M5 10h14 M5 14h14",
  "metro-entrance": "M5 19V9a7 7 0 0 1 14 0v10 M9 19v-6h6v6",
  "metro-station": "M5 19V9a7 7 0 0 1 14 0v10 M9 19v-6h6v6",
  "raider-camp": "M12 4L3 19h18z M12 9l-3 10 M12 9l3 10",
  "field-depot": "M4 8l8-4 8 4v8l-8 4-8-4z M4 8l8 4 8-4 M12 12v8",
  "supply-station": "M4 8l8-4 8 4v8l-8 4-8-4z M4 8l8 4 8-4 M12 12v8",
  "field-crate": "M4 7h16v12H4z M4 11h16 M10 11v3h4v-3",
  "locked-room": "M7 11V8a5 5 0 0 1 10 0v3 M5 11h14v9H5z M12 14v3",
  "breach-room": "M5 4h10l4 4v12H5z M9 12l6 4 M15 12l-6 4",
  zipline: "M3 6l18 7 M9 8v7 M7 15h4",
  "mast-lift": "M8 3v18 M16 3v18 M8 7h8 M8 12h8 M8 17h8",
  "gondola-landing": "M3 5l18 4 M12 7v4 M7 11h10v8H7z",
  "player-spawn": "M12 4a2.5 2.5 0 1 1 0 5a2.5 2.5 0 1 1 0-5 M7 21l2-8h6l2 8 M8 13l-2 3 M16 13l2 3",
  "fuel-cell": "M8 6h8v14H8z M10 3h4v3 M12 10l-2 4h4l-2 4",
  button: "M12 7a5 5 0 1 1 0 10a5 5 0 1 1 0-10 M12 10v4",
  antenna: "M12 10v11 M8 21h8 M8 8a5 5 0 0 1 8 0 M5 5a9 9 0 0 1 14 0",
  "emergency-shelter": "M3 12l9-8 9 8 M6 10v10h12V10 M10 20v-5h4v5",
  "mountaineers-detector": "M4 19l6-12 4 7 2-3 4 8z",
  "ammo-crate": "M6 9h3v10H6z M10.5 9h3v10h-3z M15 9h3v10h-3z M6 9l1.5-4 1.5 4 M10.5 9l1.5-4 1.5 4 M15 9l1.5-4 1.5 4",
  "med-crate": "M10 5h4v5h5v4h-5v5h-4v-5H5v-4h5z",
  "weapon-case": "M3 9h18v9H3z M8 9V6h8v3 M3 13h18",
  locker: "M7 3h10v18H7z M12 3v18 M9 10v2 M15 10v2",
  "raider-cache": "M12 3l2.6 5.6 6 .7-4.5 4.1 1.2 6L12 16.4 6.7 19.4l1.2-6L3.4 9.3l6-.7z",
  "frozen-raider-cache": "M12 3l2.6 5.6 6 .7-4.5 4.1 1.2 6L12 16.4 6.7 19.4l1.2-6L3.4 9.3l6-.7z",
  "hurricane-cache": "M12 4a8 8 0 1 0 8 8 M12 8a4 4 0 1 0 4 4 M12 12h.1",
  bag: "M6 8h12l-1 12H7z M9 8a3 3 0 0 1 6 0",
  suitcase: "M4 8h16v11H4z M9 8V5h6v3",
  basket: "M4 10h16l-2 9H6z M8 10l3-5 M16 10l-3-5",
  car: "M3 15l2-6h14l2 6v4H3z M7 19v1 M17 19v1 M6 15h.1 M18 15h.1",
  box: "M4 7h16v12H4z M4 11h16",
  "base-container": "M4 7h16v12H4z M8 7v12 M12 7v12 M16 7v12",
  "industrial-container": "M4 7h16v12H4z M8 7v12 M12 7v12 M16 7v12",
  "electrical-container": "M4 7h16v12H4z M13 9l-3 5h4l-3 4",
  "breachable-container": "M4 7h16v12H4z M8 10l8 6 M16 10l-8 6",
  "utility-crate": "M4 7h16v12H4z M9 13h6 M12 10v6",
  "combat-supplies": "M4 7h16v12H4z M12 9v8 M8 13h8",
  "security-breach": "M12 3l8 4v5c0 5-4 8-8 9-4-1-8-4-8-9V7z M12 8v5 M12 16v.1",
  android: "M8 8h8v10H8z M10 5v3 M14 5v3 M10 12h.1 M14 12h.1",
  quests: "M12 4v10 M12 18v.5",
  bird: "M3 12c4-4 7-4 9 0 2-4 5-4 9 0 M12 12v6",
  "snow-pile": "M4 18c2-5 5-8 8-8s6 3 8 8z M8 6v3 M6.5 7.5h3 M16 4v3 M14.5 5.5h3",
  "security-camera": "M3 9l12-4 2 5-12 4z M10 13l1 6 M7 19h8",
  "dig-spot": "M12 3v12 M8 11l4 4 4-4 M5 19h14",
  harvester: "M5 18l7-14 7 14z M9 14h6",
  mine: "M12 7a5 5 0 1 1 0 10a5 5 0 1 1 0-10 M12 3v3 M12 18v3 M3 12h3 M18 12h3",
  "car-alarm": "M3 15l2-6h14l2 6v4H3z M12 3v3 M7 5l1.5 2 M17 5l-1.5 2",
  piano: "M4 7h16v12H4z M8 7v8 M12 7v8 M16 7v8",
};

const GROUP_GLYPH: Record<string, string> = {
  locations: "M12 3a6 6 0 0 1 6 6c0 5-6 12-6 12S6 14 6 9a6 6 0 0 1 6-6z M12 7.5a1.5 1.5 0 1 1 0 3a1.5 1.5 0 1 1 0-3",
  quests: "M12 4v10 M12 18v.5",
  containers: "M4 8h16v11H4z M4 12h16 M10 12v3h4v-3",
  arc: "M12 3l9 9-9 9-9-9z M9 11h.1 M15 11h.1 M9 15h6",
  events: "M13 2L5 14h6l-1 8 8-12h-6z",
  nature: "M5 19c0-9 6-14 14-14 0 8-5 14-14 14z M5 19l8-8",
};

const glyphFor = (id: string, group: string) => {
  const sub = id.split("/")[1]?.replace(/_/g, "-") ?? id;
  return GLYPHS[sub] ?? (group === "arc" ? GROUP_GLYPH.arc : GROUP_GLYPH[group] ?? GROUP_GLYPH.locations);
};

const sprites = new Map<string, HTMLCanvasElement>();

/** A badge for this marker type at this size in CSS pixels, drawn for the screen's density. */
export function sprite(id: string, group: string, size: number, ratio = devicePixelRatio || 1): HTMLCanvasElement {
  const key = `${id}|${size}|${ratio}`;
  const cached = sprites.get(key);
  if (cached) return cached;
  const px = Math.ceil(size * ratio);
  const c = document.createElement("canvas");
  c.width = c.height = px;
  const ctx = c.getContext("2d")!;
  ctx.scale(px / 24, px / 24);
  ctx.beginPath();
  ctx.arc(12, 12, 11, 0, Math.PI * 2);
  ctx.fillStyle = GROUPS[group]?.color ?? "#888";
  ctx.fill();
  ctx.lineWidth = 1.4;
  ctx.strokeStyle = "rgba(10, 12, 18, 0.85)";
  ctx.stroke();
  ctx.translate(4.8, 4.8);
  ctx.scale(0.6, 0.6);
  ctx.strokeStyle = "#fff";
  ctx.lineWidth = 2.4;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  ctx.stroke(new Path2D(glyphFor(id, group)));
  sprites.set(key, c);
  return c;
}

const urls = new Map<string, string>();

/** The same icon as an image address, for the legend. */
export function iconUrl(id: string, group: string): string {
  if (!urls.has(id)) urls.set(id, sprite(id, group, 20, 2).toDataURL());
  return urls.get(id)!;
}
