// Report relay: the app POSTs a report here, and this files it on the data
// repository as the GitHub App's bot. The bot's key never leaves Cloudflare.

import { b64, GitHub, type GitHubEnv } from "./github.ts";
import { correction, issueBody, issueTitle, validate } from "./report.ts";

export interface Env extends GitHubEnv {
  /** Workers rate-limit binding, keyed per IP. Absent in tests. */
  REPORTS?: { limit(o: { key: string }): Promise<{ success: boolean }> };
}

const MAX_BODY = 16 * 1024;

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });
}

export async function handle(req: Request, env: Env): Promise<Response> {
  const url = new URL(req.url);
  if (url.pathname === "/health") return json(200, { ok: true });
  if (url.pathname !== "/report") return json(404, { error: "not found" });
  if (req.method !== "POST") return json(405, { error: "POST only" });
  if (!(req.headers.get("user-agent") ?? "").startsWith("arc-companion/")) return json(403, { error: "unknown client" });

  const ip = req.headers.get("cf-connecting-ip") ?? "unknown";
  if (env.REPORTS && !(await env.REPORTS.limit({ key: ip })).success) {
    return json(429, { error: "Too many reports at once. Try again in a minute." });
  }

  const raw = await req.text();
  if (raw.length > MAX_BODY) return json(413, { error: "report is too large" });
  let body: unknown;
  try {
    body = JSON.parse(raw);
  } catch {
    return json(400, { error: "not JSON" });
  }
  const checked = validate(body);
  if ("errors" in checked) return json(400, { error: checked.errors.join("; ") });
  const report = checked.report;

  const gh = await GitHub.connect(env);
  const labels = ["report", report.type === "incorrect" ? "incorrect-data" : "suggestion"];
  const issue = await gh.req<{ number: number; html_url: string }>("POST", "/issues", {
    title: issueTitle(report),
    body: issueBody(report),
    labels,
  });

  let pr: string | null = null;
  const fix = correction(report, issue.number);
  if (fix) {
    try {
      pr = await openPr(gh, issue.number, fix);
    } catch (e) {
      // The issue is the record; a failed PR just means fixing it by hand.
      console.log(`PR for #${issue.number} failed: ${e}`);
    }
  }
  return json(201, { ok: true, issue: issue.number, issue_url: issue.html_url, pr_url: pr });
}

async function openPr(gh: GitHub, n: number, fix: object): Promise<string> {
  const repo = await gh.req<{ default_branch: string }>("GET", "");
  const base = repo.default_branch;
  const head = await gh.req<{ object: { sha: string } }>("GET", `/git/ref/heads/${base}`);
  const branch = `report/${n}`;
  await gh.req("POST", "/git/refs", { ref: `refs/heads/${branch}`, sha: head.object.sha });
  await gh.req("PUT", `/contents/overlays/corrections/${n}.json`, {
    message: `Correction from report #${n}`,
    content: b64(JSON.stringify(fix, null, 2) + "\n"),
    branch,
  });
  const pr = await gh.req<{ number: number; html_url: string }>("POST", "/pulls", {
    title: `Correction from report #${n}`,
    head: branch,
    base,
    body: `Proposed by a player in #${n}. The report-pr workflow rebuilds \`data/\` into this PR.\n\nMerge if it checks out in game; closing this PR leaves the issue open.\n\nCloses #${n}`,
  });
  await gh.req("POST", `/issues/${pr.number}/labels`, { labels: ["report", "correction"] });
  return pr.html_url;
}

export default {
  async fetch(req: Request, env: Env): Promise<Response> {
    try {
      return await handle(req, env);
    } catch (e) {
      console.log(`report failed: ${e}`);
      return json(502, { error: "Couldn't file the report right now. Try again later." });
    }
  },
};
