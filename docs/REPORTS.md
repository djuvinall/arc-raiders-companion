# Player reports

Players right-click anything in the app and choose **Report…**. They pick **Incorrect data** or
**Suggestion**, optionally type what it should be and a comment, and press Send. They need no
account and see nothing of GitHub.

## How a report travels

1. **The app** (`crates/arc-tauri/src/report.rs`) looks up the reported field's current value
   and posts the report to the relay. It sends the form, the app version, the dataset version and
   the game patch, and nothing about the player.
2. **The relay** (`relay/`, a Cloudflare Worker) checks the report and limits each IP to three
   reports a minute. It then files it on arc-raiders-data as the GitHub App's bot:
   - Every report becomes an issue labelled `report` plus `incorrect-data` or `suggestion`.
   - An incorrect-data report that names a field and gives a value also gets a PR. The PR adds
     `overlays/corrections/<issue>.json` on a `report/<issue>` branch.
3. **The report-pr workflow** in arc-raiders-data rebuilds `data/` from that correction, runs
   the tests, and pushes the rebuilt data into the same PR. The PR then shows the real change.
4. **You review.** Merge if it checks out; the next data update ships it to every player.
   Merging closes the issue. Close the PR without merging to reject it; the issue stays open.

Which fact a report is about comes from `data-report="file/id[/field]"` attributes in the pages,
and its label from `data-report-label`. Untagged spots still work: the report carries the page
and the highlighted text, but it never gets a PR.

The bot's key lives only in Cloudflare. It is never in the app, where anyone could extract it.

## One-time setup

You do this once. Steps 1 to 3 need your own logins, so only you can do them.

### 1. Log Wrangler in to Cloudflare

Wrangler is Cloudflare's command-line tool for publishing Workers. `wrangler login` connects
it to your Cloudflare account, the same account your Tunnel uses. It's the only Cloudflare login
this needs. The Cloudflare MCP connector is optional, and nothing here uses it.

In a terminal (PowerShell is fine):

```
cd C:\LocalRepositories\ArcDatabase\relay
npm install
npx wrangler login
```

A browser tab opens on Cloudflare asking to allow Wrangler. Sign in if asked, then click
**Allow**. Back in the terminal, check it worked:

```
npx wrangler whoami
```

It should print your email and account.

### 2. Create the GitHub App (the bot)

1. Open <https://github.com/settings/apps/new>.
2. **GitHub App name:** `ARC Companion Reports`. The name must be unique across GitHub, so add
   something if it's taken. Issues will show as `<name>[bot]`.
3. **Homepage URL:** `https://github.com/djuvinall/arc-raiders-companion`
4. **Webhook:** untick **Active**.
5. **Repository permissions:**
   - Contents: **Read and write**
   - Issues: **Read and write**
   - Pull requests: **Read and write**
   - Metadata: Read-only (set automatically)
6. **Where can this GitHub App be installed?** Only on this account.
7. Click **Create GitHub App**.
8. On the page that opens, note the **App ID** (a number near the top).
9. Scroll to **Private keys** and click **Generate a private key**. A `.pem` file downloads.
   Treat it like a password.
10. In the left menu click **Install App**, then **Install** next to your account. Choose
    **Only select repositories**, pick **arc-raiders-data**, and click **Install**.

### 3. Publish the relay and give it the bot's key

Still in `relay\`:

```
npx wrangler deploy
```

The first deploy may ask you to pick a `workers.dev` subdomain; any name works. It ends by
printing the relay's address, like `https://arc-companion-reports.<you>.workers.dev`. Send that
address to Claude: it goes into `REPORT_URL` in `crates/arc-tauri/src/report.rs`.

Then store the two secrets. Replace the path with where the `.pem` downloaded:

```
npx wrangler secret put GITHUB_APP_ID
```

Paste the App ID when asked.

```
Get-Content "$HOME\Downloads\arc-companion-reports.2026-11-08.private-key.pem" -Raw | npx wrangler secret put GITHUB_PRIVATE_KEY
```

Then move the `.pem` somewhere safe or delete it. You can generate a new key on the App page
anytime.

Check the relay is up:

```
curl.exe https://arc-companion-reports.arc-companion-reports.workers.dev/health
```

It should print `{"ok":true}`.

### 4. Push both repositories

The data repo's workflows check out the app repo's `main` to run ingest. Push the app repo
(with `crates/arc-ingest/src/corrections.rs`) before relying on report PRs.

## Day to day

- **Reviewing:** issues and PRs on arc-raiders-data, labelled `report`.
- **Retiring a correction:** once the sources agree with it, delete its file in
  `overlays/corrections/`. Ingest prints a note when a correction's `was` no longer matches the
  sources.
- **Spam:**
  - **Stop it entirely:** suspend the App (GitHub, Settings, Applications), or run
    `npx wrangler delete` in `relay\`.
  - **Change the per-IP limit:** edit `[[ratelimits]]` in `relay/wrangler.toml`.
- **Testing locally:** `cd relay; npm test` runs the relay's tests against a fake GitHub. Set
  `ARC_REPORT_URL` to point a dev build of the app at a different relay.
