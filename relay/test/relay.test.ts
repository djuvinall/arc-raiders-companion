import assert from "node:assert/strict";
import { generateKeyPairSync, createPublicKey, verify } from "node:crypto";
import { test } from "node:test";
import { appJwt } from "../src/github.ts";
import { handle } from "../src/index.ts";
import { coerce, correction, issueBody, validate, type Report } from "../src/report.ts";

const { privateKey } = generateKeyPairSync("rsa", {
  modulusLength: 2048,
  privateKeyEncoding: { type: "pkcs1", format: "pem" },
  publicKeyEncoding: { type: "spki", format: "pem" },
});
const env = { GITHUB_APP_ID: "123", GITHUB_PRIVATE_KEY: privateKey, GITHUB_REPO: "o/data", GITHUB_API: "https://gh.test" };

const base = {
  type: "incorrect",
  ref: "items/anvil-i/value",
  label: "Anvil I · Sell value",
  page: "#/item/anvil-i",
  selected: "5,000",
  current: 5000,
  suggestion: "5,200",
  comment: null,
  app_version: "0.7.0",
  dataset_version: "2026-11-06",
  game_patch: "2.4.1",
};

function ok(b: unknown): Report {
  const r = validate(b);
  assert.ok("report" in r, JSON.stringify(r));
  return r.report;
}

test("validate accepts a full report and refuses junk", () => {
  ok(base);
  ok({ type: "suggestion", ref: "arcs/hydra", comment: "Add a tip" });
  for (const bad of [
    null,
    { ...base, type: "spam" },
    { ...base, ref: "../../etc" },
    { ...base, ref: "secrets/x/y" },
    { ...base, comment: "x".repeat(3000) },
    { type: "incorrect" },
    { type: "suggestion", ref: "arcs/hydra" },
  ]) assert.ok("errors" in validate(bad), JSON.stringify(bad));
});

test("coerce keeps the field's type", () => {
  assert.deepEqual(coerce("5,200", 5000), { ok: true, value: 5200 });
  assert.deepEqual(coerce("90 %", 80), { ok: true, value: 90 });
  assert.deepEqual(coerce("lots", 80), { ok: false });
  assert.deepEqual(coerce("Heavy", "Light"), { ok: true, value: "Heavy" });
  assert.deepEqual(coerce("3000", null), { ok: true, value: 3000 });
  assert.deepEqual(coerce("x", ["a"]), { ok: false });
});

test("correction only for concrete incorrect-data reports", () => {
  const c = correction(ok(base), 12);
  assert.deepEqual(c, { file: "items", id: "anvil-i", field: "/value", value: 5200, was: 5000, report: 12 });
  assert.equal(correction(ok({ ...base, ref: "items/anvil-i" }), 1), null);
  assert.equal(correction(ok({ ...base, suggestion: null, comment: "wrong" }), 1), null);
  assert.equal(correction(ok({ ...base, suggestion: "5000" }), 1), null);
  assert.equal(correction(ok({ ...base, type: "suggestion" }), 1), null);
  assert.equal(correction(ok({ ...base, ref: "items/heavy-shield/stats/charge", current: 80, suggestion: "90" }), 1)?.field, "/stats/charge");
});

test("issue body fences player text so it can't ping or link", () => {
  const body = issueBody(ok({ ...base, comment: "@djuvinall ~~~ see https://evil.test" }));
  assert.match(body, /~~~~\n@djuvinall ~~~ see https:\/\/evil.test\n~~~~/);
});

test("the app JWT verifies against the key", async () => {
  const jwt = await appJwt(env, 1_700_000_000);
  const [h, c, s] = jwt.split(".");
  assert.equal(JSON.parse(Buffer.from(c, "base64url").toString()).iss, "123");
  const pub = createPublicKey(privateKey);
  assert.ok(verify("sha256", Buffer.from(`${h}.${c}`), pub, Buffer.from(s, "base64url")));
});

test("a report files an issue and a PR as the app", async () => {
  const calls: { method: string; path: string; body: any }[] = [];
  globalThis.fetch = (async (url: string, init: RequestInit) => {
    const path = url.replace("https://gh.test", "");
    const body = init.body ? JSON.parse(init.body as string) : undefined;
    calls.push({ method: init.method!, path, body });
    const reply: Record<string, unknown> = {
      "GET /repos/o/data/installation": { id: 9 },
      "POST /app/installations/9/access_tokens": { token: "t" },
      "POST /repos/o/data/issues": { number: 12, html_url: "https://github.com/o/data/issues/12" },
      "GET /repos/o/data": { default_branch: "main" },
      "GET /repos/o/data/git/ref/heads/main": { object: { sha: "abc" } },
      "POST /repos/o/data/git/refs": {},
      "PUT /repos/o/data/contents/overlays/corrections/12.json": {},
      "POST /repos/o/data/pulls": { number: 13, html_url: "https://github.com/o/data/pull/13" },
      "POST /repos/o/data/issues/13/labels": [],
    };
    const r = reply[`${init.method} ${path}`];
    return r === undefined ? new Response("nope", { status: 404 }) : new Response(JSON.stringify(r), { status: 200 });
  }) as typeof fetch;

  const req = (body: unknown, ua = "arc-companion/0.7.0") =>
    new Request("https://relay.test/report", { method: "POST", headers: { "user-agent": ua }, body: JSON.stringify(body) });

  assert.equal((await handle(req(base, "curl/8"), env)).status, 403);
  assert.equal((await handle(req({ type: "x" }), env)).status, 400);

  const res = await handle(req(base), env);
  assert.equal(res.status, 201);
  assert.deepEqual(await res.json(), {
    ok: true, issue: 12, issue_url: "https://github.com/o/data/issues/12", pr_url: "https://github.com/o/data/pull/13",
  });
  const issue = calls.find((c) => c.path === "/repos/o/data/issues")!;
  assert.deepEqual(issue.body.labels, ["report", "incorrect-data"]);
  const file = calls.find((c) => c.method === "PUT")!;
  assert.equal(file.body.branch, "report/12");
  assert.equal(JSON.parse(Buffer.from(file.body.content, "base64").toString()).value, 5200);
  assert.ok(calls.find((c) => c.path === "/repos/o/data/pulls")!.body.body.includes("Closes #12"));

  const limited = await handle(req(base), { ...env, REPORTS: { limit: async () => ({ success: false }) } });
  assert.equal(limited.status, 429);
});
