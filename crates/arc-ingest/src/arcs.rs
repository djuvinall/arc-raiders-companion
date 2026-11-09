//! ARC details from the wiki's page infoboxes (CC BY-SA).

use crate::skills::clean;
use arc_core::model::Arc;
use serde_json::Value;

fn field(wikitext: &str, key: &str) -> Option<String> {
    wikitext.lines().find_map(|line| {
        let (name, value) = line.trim_start().strip_prefix('|')?.split_once('=')?;
        (name.trim() == key).then(|| value.trim().to_string()).filter(|v| !v.is_empty())
    })
}

fn tips(wikitext: &str) -> Option<String> {
    let (_, rest) = wikitext.split_once("== Combat tips ==")?;
    let body = rest.split("\n==").next()?;
    let text: Vec<String> = body.lines()
        .map(|l| clean(l.trim_start_matches(['*', ' '])).replace("'''", "").replace("''", ""))
        .filter(|l| !l.is_empty() && !l.starts_with("{{") && !l.starts_with('['))
        .collect();
    (!text.is_empty()).then(|| text.join(" "))
}

pub fn apply(arcs: &mut [Arc], pages: &Value) {
    for arc in arcs {
        let Some(page) = pages[arc.id.as_str()].as_str() else { continue };
        let plain = |key: &str| field(page, key)
            .map(|v| v.split("<br>").map(|p| clean(p.trim_end_matches('/'))).filter(|p| !p.is_empty()).collect::<Vec<_>>().join(", "))
            .filter(|v| !v.is_empty() && v != "None");
        if let Some(image) = field(page, "image") {
            arc.icon = Some(format!("https://arcraiders.wiki/wiki/Special:FilePath/{}", image.replace(' ', "_")));
        }
        arc.hp = field(page, "health").and_then(|h| h.replace(',', "").split_whitespace().next()?.parse().ok());
        arc.armor = plain("armor");
        arc.attack = plain("pAttack");
        arc.ability = plain("ability");
        arc.weak_points = field(page, "weakness").map_or(Vec::new(), |w| {
            w.split("<br>").map(|p| clean(p.trim_end_matches('/'))).filter(|p| !p.is_empty()).collect()
        });
        arc.tips = tips(page);
    }
}

#[derive(serde::Deserialize)]
struct TipFixes {
    fixes: Vec<TipFix>,
}

#[derive(serde::Deserialize)]
struct TipFix {
    arc: String,
    find: String,
    replace: String,
}

/// Rewrites tip sentences that patches made stale. A fix whose sentence is gone is
/// reported, so a wiki edit can retire it.
pub fn fix_tips(arcs: &mut [Arc], overlay: &str) -> anyhow::Result<()> {
    let fixes: TipFixes = serde_json::from_str(overlay)?;
    for fix in &fixes.fixes {
        let Some(arc) = arcs.iter_mut().find(|a| a.id == fix.arc) else {
            println!("note: tip_fixes: no ARC {}", fix.arc);
            continue;
        };
        match arc.tips.as_mut().filter(|t| t.contains(&fix.find)) {
            Some(tips) => {
                *tips = tips.replace(&fix.find, &fix.replace).replace("  ", " ").trim().to_string();
            }
            None => println!("note: tip_fixes: {} tips no longer say {:?}", fix.arc, fix.find),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_infobox_and_tips() {
        let page = "{{Infobox arc\n| armor = Heavy\n| weakness = Rear plate<br>[[Leg]] joints\n| health = 2,050\n}}\nIntro\n== Combat tips ==\nShoot the '''yellow''' parts.\n\n== Loot ==\nX";
        assert_eq!(field(page, "health").as_deref(), Some("2,050"));
        assert_eq!(tips(page).as_deref(), Some("Shoot the yellow parts."));
    }
}
