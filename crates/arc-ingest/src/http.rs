//! Polite blocking HTTP: fixed user agent, a delay between calls, bounded retries.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::thread::sleep;
use std::time::Duration;

const USER_AGENT: &str = "arc-companion-ingest/0.1 (open-source ARC Raiders companion)";

pub struct Client {
    agent: ureq::Agent,
    delay: Duration,
}

impl Client {
    pub fn new(delay_ms: u64) -> Self {
        let agent = ureq::AgentBuilder::new()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(60))
            .build();
        Self { agent, delay: Duration::from_millis(delay_ms) }
    }

    pub fn get_json(&self, url: &str) -> Result<Value> {
        let text = self.get_text(url)?;
        serde_json::from_str(&text).with_context(|| format!("bad JSON from {url}"))
    }

    pub fn get_text(&self, url: &str) -> Result<String> {
        let mut last = String::new();
        for attempt in 0..3u64 {
            sleep(self.delay + Duration::from_secs(attempt * 2));
            match self.agent.get(url).call() {
                Ok(resp) => return resp.into_string().context("reading body"),
                Err(ureq::Error::Status(code, _)) if code < 500 && code != 429 => {
                    bail!("{url}: HTTP {code}")
                }
                Err(e) => last = e.to_string(),
            }
        }
        bail!("{url}: gave up after retries: {last}")
    }
}
