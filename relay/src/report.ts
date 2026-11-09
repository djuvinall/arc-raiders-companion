// Pure parts of the relay: checking a report and turning it into an issue and,
// when it names a field and a value, a correction file. No network here.

export const FILES = [
  "items", "recipes", "workbenches", "weapon_mods", "maps", "pois", "quests",
  "projects", "skills", "arcs", "traders", "loot_areas",
] as const;

export type ReportType = "incorrect" | "suggestion";

export interface Report {
  type: ReportType;
  /** `file/id` or `file/id/field/...`, as the app tags what was reported. */
  ref: string | null;
  /** What the player saw it as, e.g. "Anvil I · Sell value". */
  label: string | null;
  page: string | null;
  selected: string | null;
  /** The field's raw value in the player's dataset, when ref names a field. */
  current?: unknown;
  suggestion: string | null;
  comment: string | null;
  app_version: string | null;
  dataset_version: string | null;
  game_patch: string | null;
}

export interface Correction {
  file: string;
  id: string;
  field: string;
  value: unknown;
  was: unknown;
  report: number;
  comment?: string;
}

const LIMITS: Record<string, number> = {
  label: 200, page: 200, selected: 1000, suggestion: 500, comment: 2000,
  app_version: 40, dataset_version: 40, game_patch: 40,
};
const REF = /^([a-z_]+)\/([a-z0-9][a-z0-9-]{0,99})(\/[A-Za-z0-9_-]{1,60}){0,4}$/;

function text(v: unknown, name: string, errors: string[]): string | null {
  if (v === undefined || v === null) return null;
  if (typeof v !== "string") {
    errors.push(`${name} must be text`);
    return null;
  }
  const t = v.trim();
  if (t.length > LIMITS[name]) errors.push(`${name} is too long`);
  return t || null;
}

/** Returns the cleaned report, or the reasons it was refused. */
export function validate(body: unknown): { report: Report } | { errors: string[] } {
  const errors: string[] = [];
  if (!body || typeof body !== "object") return { errors: ["not a report"] };
  const b = body as Record<string, unknown>;
  if (b.type !== "incorrect" && b.type !== "suggestion") errors.push("type must be incorrect or suggestion");
  let ref: string | null = null;
  if (b.ref !== undefined && b.ref !== null) {
    if (typeof b.ref !== "string" || !REF.test(b.ref) || !(FILES as readonly string[]).includes(b.ref.split("/")[0])) {
      errors.push("ref is not a dataset path");
    } else ref = b.ref;
  }
  const report: Report = {
    type: b.type as ReportType,
    ref,
    label: text(b.label, "label", errors),
    page: text(b.page, "page", errors),
    selected: text(b.selected, "selected", errors),
    current: b.current,
    suggestion: text(b.suggestion, "suggestion", errors),
    comment: text(b.comment, "comment", errors),
    app_version: text(b.app_version, "app_version", errors),
    dataset_version: text(b.dataset_version, "dataset_version", errors),
    game_patch: text(b.game_patch, "game_patch", errors),
  };
  if (!report.ref && !report.selected) errors.push("say what the report is about");
  if (report.type === "suggestion" && !report.suggestion && !report.comment) errors.push("a suggestion needs some text");
  if (JSON.stringify(b.current ?? null).length > 2000) errors.push("current is too long");
  return errors.length ? { errors } : { report };
}

/** User text goes in a fence so it can't ping people, link, or render markup. */
function fence(s: string): string {
  const longest = Math.max(2, ...(s.match(/~+/g) ?? []).map((m) => m.length));
  const f = "~".repeat(longest + 1);
  return `${f}\n${s}\n${f}`;
}

function short(s: string, n: number): string {
  const one = s.replace(/\s+/g, " ");
  return one.length > n ? one.slice(0, n - 1) + "…" : one;
}

export function issueTitle(r: Report): string {
  const what = r.label ?? r.ref ?? short(r.selected ?? "", 50);
  return r.type === "incorrect" ? `Incorrect data: ${short(what, 80)}` : `Suggestion: ${short(what, 80)}`;
}

export function issueBody(r: Report): string {
  const lines: string[] = [];
  lines.push(r.type === "incorrect" ? "A player reported this as incorrect." : "A player sent a suggestion.", "");
  if (r.ref) lines.push(`- **Dataset path:** \`${r.ref}\``);
  if (r.page) lines.push(`- **App page:** \`${r.page.replace(/`/g, "")}\``);
  if (r.label) lines.push("", "**What it was shown as:**", fence(r.label));
  if (r.current !== undefined && r.current !== null) lines.push("", "**Current value in the dataset:**", fence(JSON.stringify(r.current)));
  if (r.selected) lines.push("", "**Highlighted text:**", fence(r.selected));
  if (r.suggestion) lines.push("", r.type === "incorrect" ? "**Suggested value:**" : "**Suggestion:**", fence(r.suggestion));
  if (r.comment) lines.push("", "**Comment:**", fence(r.comment));
  lines.push("", "---", `App ${r.app_version ?? "?"} · dataset ${r.dataset_version ?? "?"} · game patch ${r.game_patch ?? "?"}`);
  lines.push("", "<sub>Sent from the app's Report window. The reporter is anonymous.</sub>");
  return lines.join("\n");
}

/** Reads the suggestion as the same kind of value as the current one, or gives up. */
export function coerce(suggestion: string, current: unknown): { ok: true; value: unknown } | { ok: false } {
  const s = suggestion.trim();
  if (typeof current === "number" || current === null) {
    const n = Number(s.replace(/[,\s_]/g, "").replace(/(kg|%|hp|s)$/i, ""));
    if (Number.isFinite(n) && /\d/.test(s)) return { ok: true, value: n };
    return typeof current === "number" ? { ok: false } : { ok: true, value: s };
  }
  if (typeof current === "boolean") {
    if (/^(true|yes)$/i.test(s)) return { ok: true, value: true };
    if (/^(false|no)$/i.test(s)) return { ok: true, value: false };
    return { ok: false };
  }
  if (typeof current === "string") return { ok: true, value: s };
  return { ok: false }; // lists and objects: leave it to the issue
}

/** A correction file when the report is concrete enough to merge, else null. */
export function correction(r: Report, issue: number): Correction | null {
  if (r.type !== "incorrect" || !r.ref || !r.suggestion || r.current === undefined) return null;
  const [file, id, ...field] = r.ref.split("/");
  if (!field.length) return null;
  const v = coerce(r.suggestion, r.current);
  if (!v.ok || JSON.stringify(v.value) === JSON.stringify(r.current)) return null;
  const c: Correction = { file, id, field: "/" + field.join("/"), value: v.value, was: r.current, report: issue };
  if (r.comment) c.comment = r.comment;
  return c;
}
