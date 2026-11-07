//! From the wiki's infoboxes (CC BY-SA): weapon fire rate in RPM, headshot multiplier
//! and reload time, and shield charge and damage reduction. Every level of a gun shares
//! one page; amplified versions have none, so they keep empty measured values. The wiki
//! is edited after each patch, so its shield numbers win over the API's (the Heavy
//! Shield's charge went from 80 to 90 in 2.3.0 and MetaForge still had 80).

use crate::fetch::weapon_family;
use arc_core::model::{Category, Item, Provenance, Stats};
use serde_json::Value;

/// An infobox value with HTML comments and refs removed.
fn field(wikitext: &str, key: &str) -> Option<String> {
    wikitext.lines().find_map(|line| {
        let (name, value) = line.trim_start().strip_prefix('|')?.split_once('=')?;
        (name.trim() == key).then(|| strip_markup(value).trim().to_string()).filter(|v| !v.is_empty())
    })
}

fn strip_markup(text: &str) -> String {
    let mut out = text.to_string();
    for (open, close) in [("<!--", "-->"), ("<ref", ">")] {
        while let Some(start) = out.find(open) {
            let Some(len) = out[start..].find(close) else { out.truncate(start); break };
            out.replace_range(start..start + len + close.len(), "");
        }
    }
    out
}

/// The first number in a value, ignoring thousands commas: "2.0x" → 2.0.
fn number(text: &str) -> Option<f64> {
    let start = text.find(|c: char| c.is_ascii_digit())?;
    let digits: String = text[start..].chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',').filter(|c| *c != ',').collect();
    digits.trim_end_matches('.').parse().ok()
}

/// "33.3 (500 RPM)" → 500.
fn rpm(text: &str) -> Option<f64> {
    let (before, _) = text.split_once("RPM")?;
    number(&before[before.rfind('(').map_or(0, |i| i + 1)..])
}

pub fn apply(items: &mut [Item], pages: &Value) {
    for item in items.iter_mut().filter(|i| i.category == Category::Shield) {
        let Some(page) = pages[item.name.as_str()].as_str() else { continue };
        let Some(Stats::Shield(stats)) = item.stats.as_mut() else { continue };
        if let Some(charge) = field(page, "charge").as_deref().and_then(number) {
            stats.charge = Some(charge);
        }
        if let Some(reduction) = field(page, "damageM").as_deref().and_then(number) {
            stats.damage_mitigation = Some(reduction);
        }
    }
    for item in items.iter_mut().filter(|i| i.category == Category::Weapon && !i.name.starts_with("Amplified")) {
        let Some(page) = pages[weapon_family(&item.name)].as_str() else { continue };
        let Some(Stats::Weapon(stats)) = item.stats.as_mut() else { continue };
        let m = &mut stats.measured;
        m.rounds_per_minute = field(page, "firerate").as_deref().and_then(rpm);
        m.headshot_mult = field(page, "headshotmultiplier").as_deref().and_then(number);
        m.reload_s = field(page, "reloadtime").as_deref().and_then(number);
        if m.rounds_per_minute.is_some() || m.headshot_mult.is_some() || m.reload_s.is_some() {
            m.measured_provenance = Some(Provenance::Wiki);
            m.measured_by = Some("arcraiders.wiki".into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rpm_and_headshot_from_the_infobox() {
        let page = "{{Infobox weapon\n| firerate = 33.3 (500 RPM)\n| reloadtime = \n| headshotmultiplier = 2.0x\n}}";
        assert_eq!(field(page, "firerate").as_deref().and_then(rpm), Some(500.0));
        assert_eq!(field(page, "headshotmultiplier").as_deref().and_then(number), Some(2.0));
        assert_eq!(field(page, "reloadtime"), None);
        assert_eq!(rpm("1,200 RPM"), Some(1200.0));
        let messy = "| reloadtime = 4.5s {{!}} ?<!--(4.5*0.111)=4.0005s-->
| firerate = 9 <ref name=\"stats\"/>
| other = <!-- x -->";
        assert_eq!(field(messy, "reloadtime").as_deref().and_then(number), Some(4.5));
        assert_eq!(field(messy, "firerate").as_deref().and_then(number), Some(9.0));
        assert_eq!(field(messy, "other"), None);
    }
}
