# Report relay

A Cloudflare Worker that receives player reports from the app and files them on
arc-raiders-data as a GitHub App's bot. It holds the bot's key, so the app never ships one.

Setup, the report flow, and day-to-day use are in [docs/REPORTS.md](../docs/REPORTS.md).

```
npm install
npm test            # unit tests and a full report against a fake GitHub
npx wrangler deploy
```
