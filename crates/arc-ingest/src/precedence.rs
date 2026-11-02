//! Field precedence from precedence.toml. `pick` takes the first non-null value
//! in the declared source order.

use anyhow::{Context, Result};
use std::collections::BTreeMap;

pub struct Precedence(BTreeMap<String, Vec<String>>);

impl Precedence {
    pub fn parse(text: &str) -> Result<Self> {
        Ok(Self(toml::from_str(text).context("precedence.toml")?))
    }

    pub fn order(&self, field: &str) -> &[String] {
        self.0.get(field).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn pick<T: Clone>(&self, field: &str, candidates: &[(&str, Option<T>)]) -> Option<T> {
        self.order(field).iter().find_map(|source| {
            candidates.iter().find(|(s, _)| s == source).and_then(|(_, v)| v.clone())
        })
    }
}
