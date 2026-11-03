//! Fixtures -> canonical dataset. Deterministic: same fixtures, same bytes.
//! Anything that cannot be resolved is dropped and listed in coverage.json.

use crate::precedence::Precedence;
use anyhow::{Context, Result};
use arc_core::model::*;
use arc_core::DatasetFiles;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct Sources {
    pub date: String,
    pub metaforge: Value,
    pub wiki: Value,
    pub raidtheory: Value,
    pub skills_wikitext: String,
}

pub const MAPS: &[(&str, &str)] = &[
    ("dam", "Dam Battlegrounds"),
    ("spaceport", "The Spaceport"),
    ("buried-city", "Buried City"),
    ("blue-gate", "Blue Gate"),
    ("stella-montis", "Stella Montis"),
    ("riven-tides", "Riven Tides"),
    ("pendola-pass", "Pendola Pass"),
];

const ROMAN: &[(&str, u8)] = &[("I", 1), ("II", 2), ("III", 3), ("IV", 4)];

pub fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in decode(name).to_lowercase().chars() {
        if c == '\'' || c == '’' || c == '"' {
            continue;
        }
        if c.is_ascii_alphanumeric() {
            if dash && !out.is_empty() {
                out.push('-');
            }
            dash = false;
            out.push(c);
        } else {
            dash = true;
        }
    }
    out
}

fn decode(s: &str) -> String {
    s.replace("&#39;", "'").replace("&amp;", "&").replace("&quot;", "\"").replace('’', "'")
}

fn text(row: &Value, key: &str) -> String {
    row[key].as_str().map(|s| decode(s).trim().to_string()).unwrap_or_default()
}

fn opt_text(row: &Value, key: &str) -> Option<String> {
    Some(text(row, key)).filter(|s| !s.is_empty())
}

fn rows<'a>(doc: &'a Value, key: &str) -> &'a [Value] {
    doc[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn wiki_rows<'a>(doc: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    rows(doc, key).iter().filter(|r| !text(r, "page").starts_with("User:"))
}

fn num(row: &Value, key: &str) -> Option<f64> {
    let v = match &row[key] {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    };
    v.filter(|n| *n != 0.0)
}

fn count(row: &Value, key: &str) -> Option<u32> {
    num(row, key).map(|n| n.round() as u32)
}

fn stat_text(row: &Value, key: &str) -> Option<String> {
    match &row[key] {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

/// Splits a trailing tier numeral: "Rattler III" -> ("Rattler", Some(3)).
fn split_tier(name: &str) -> (&str, Option<u8>) {
    if let Some((head, tail)) = name.rsplit_once(' ') {
        if let Some((_, tier)) = ROMAN.iter().find(|(r, _)| *r == tail) {
            return (head, Some(*tier));
        }
    }
    (name, None)
}

/// "Rattler III" and "Aphelion Rifle" both reduce to the family mods are listed under.
fn weapon_family(name: &str) -> String {
    let (family, _) = split_tier(name);
    slug(family).trim_end_matches("-rifle").to_string()
}

fn mk_tier(name: &str) -> Option<u8> {
    let (_, rest) = name.split_once("Mk. ")?;
    rest.chars().next()?.to_digit(10).map(|d| d as u8)
}

fn parse_rarity(s: &str) -> Option<Rarity> {
    match s.to_lowercase().as_str() {
        "common" => Some(Rarity::Common),
        "uncommon" => Some(Rarity::Uncommon),
        "rare" => Some(Rarity::Rare),
        "epic" => Some(Rarity::Epic),
        "legendary" => Some(Rarity::Legendary),
        _ => None,
    }
}

fn split_areas(s: &str) -> Vec<String> {
    let set: BTreeSet<String> = s.split(',').map(slug).filter(|a| !a.is_empty()).collect();
    set.into_iter().collect()
}

fn category(item_type: &str, name: &str, block: &Value, cosmetic: bool) -> Category {
    if cosmetic {
        return Category::Cosmetic;
    }
    match item_type {
        "Weapon" => Category::Weapon,
        "Modification" | "Mods" => Category::WeaponMod,
        "Shield" => Category::Shield,
        "Augment" => Category::Augment,
        "Ammunition" => Category::Ammo,
        "Key" => Category::Key,
        "Basic Material" | "Material" => Category::MaterialBasic,
        "Topside Material" => Category::MaterialTopside,
        "Refined Material" => Category::MaterialRefined,
        "Recyclable" => Category::Recyclable,
        "Nature" => Category::Nature,
        "Trinket" => Category::Trinket,
        "Research" | "Research Item" => Category::Research,
        "Quest Item" => Category::QuestItem,
        "Blueprint" => Category::Blueprint,
        "Stencil" => Category::Stencil,
        "Design" => Category::Design,
        "Furniture" => Category::Furniture,
        "Cosmetic" | "Outfits" => Category::Cosmetic,
        "Quick Use" => {
            let lower = name.to_lowercase();
            if lower.contains("grenade") || lower.contains("'nade") || lower.ends_with(" mine") {
                Category::Grenade
            } else if ["healing", "healingPerSecond", "stamina", "shieldCharge", "healCapacity"]
                .iter()
                .any(|k| num(block, k).is_some())
            {
                Category::Healing
            } else {
                Category::Gadget
            }
        }
        _ => Category::Misc,
    }
}

fn project_stats(cat: Category, block: &Value, shield_type: Option<String>, slots: Vec<ModSlot>) -> Option<Stats> {
    match cat {
        Category::Weapon => Some(Stats::Weapon(WeaponStats {
            damage: num(block, "damage"),
            fire_rate: num(block, "fireRate"),
            range: num(block, "range"),
            stability: num(block, "stability"),
            agility: num(block, "agility"),
            stealth: num(block, "stealth"),
            magazine_size: count(block, "magazineSize"),
            firing_mode: stat_text(block, "firingMode"),
            arc_armor_penetration: stat_text(block, "arcArmorPenetration").map(|s| s.to_lowercase()),
            mod_slots: slots,
            upgrades: weapon_upgrades(block),
            measured: MeasuredStats::default(),
        })),
        Category::Shield => Some(Stats::Shield(ShieldStats {
            shield_hp: None,
            charge: num(block, "shieldCharge"),
            shield_type,
            movement_penalty: num(block, "movementPenalty"),
            damage_mitigation: num(block, "damageMitigation"),
        })),
        Category::Augment => Some(Stats::Augment(AugmentStats {
            backpack_slots: count(block, "backpackSlots"),
            weight_limit_kg: num(block, "weightLimitKg").or_else(|| num(block, "weightLimit")),
            safe_pocket_slots: count(block, "safePocketSlots"),
            quick_use_slots: count(block, "quickUseSlots"),
            shield_compatibility: stat_text(block, "shieldCompatibility"),
            trinket_slots: count(block, "trinketSlots"),
        })),
        Category::Healing | Category::Gadget | Category::Grenade | Category::Tool => {
            let s = ConsumableStats {
                healing: num(block, "healing"),
                healing_per_second: num(block, "healingPerSecond"),
                use_time: num(block, "useTime"),
                duration: num(block, "duration"),
                radius: num(block, "radius"),
                arc_stun: num(block, "arcStun"),
                raider_stun: num(block, "raiderStun"),
            };
            let empty = s.healing.is_none() && s.healing_per_second.is_none()
                && s.use_time.is_none() && s.duration.is_none() && s.radius.is_none()
                && s.arc_stun.is_none() && s.raider_stun.is_none();
            (!empty).then_some(Stats::Consumable(s))
        }
        _ => None,
    }
}

/// Maps source names and ids to canonical ids, and records what it could not.
struct Resolver {
    by_name: HashMap<String, String>,
    by_mf_id: HashMap<String, String>,
    unresolved: BTreeSet<String>,
}

impl Resolver {
    fn find(&self, name: &str) -> Option<String> {
        let n = decode(name).trim().to_lowercase();
        let base = n.strip_suffix(" i").unwrap_or(&n);
        [n.clone(), base.to_string(), format!("{n} i"), format!("{base} rifle")]
            .iter()
            .find_map(|k| self.by_name.get(k).cloned())
    }

    fn name(&mut self, context: &str, name: &str) -> Option<String> {
        let found = self.find(name);
        if found.is_none() && !name.trim().is_empty() {
            self.unresolved.insert(format!("{context}: {}", decode(name).trim()));
        }
        found
    }

    fn mf(&mut self, context: &str, id: &str) -> Option<String> {
        let found = self.by_mf_id.get(id).cloned();
        if found.is_none() {
            self.unresolved.insert(format!("{context}: metaforge id {id}"));
        }
        found
    }

    fn insert(&mut self, name: &str, id: &str) {
        self.by_name.entry(decode(name).to_lowercase()).or_insert_with(|| id.to_string());
    }
}

fn qty(row: &Value) -> u32 {
    count(row, "qty").or_else(|| count(row, "quantity")).unwrap_or(1)
}

fn merge_qty(list: &mut Vec<ItemQty>, item_id: String, n: u32) {
    match list.iter_mut().find(|q| q.item_id == item_id) {
        Some(q) => q.qty += n,
        None => list.push(ItemQty { item_id, qty: n }),
    }
}

/// `Rattler I:1:Rattler` is name:qty:page.
fn parse_ingredients(res: &mut Resolver, context: &str, s: &str) -> Option<Vec<ItemQty>> {
    let mut out = Vec::new();
    let mut ok = true;
    for part in s.split(',').filter(|p| !p.trim().is_empty()) {
        let mut bits = part.split(':');
        let name = bits.next().unwrap_or("");
        let n = bits.next().and_then(|q| q.trim().parse().ok()).unwrap_or(1);
        match res.name(context, name) {
            Some(id) => merge_qty(&mut out, id, n),
            None => ok = false,
        }
    }
    ok.then_some(out)
}

/// "Gunsmith 1", "Workbench", "Gunsmith I".
fn parse_station(raw: &str) -> Option<(String, String, Option<u8>)> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (name, level) = match raw.rsplit_once(' ') {
        Some((head, tail)) if tail.parse::<u8>().is_ok() => (head, tail.parse().ok()),
        _ => split_tier(raw),
    };
    let name = if name == "Workshop" { "Workbench" } else { name };
    let display = if name == "Inventory" { "Field Crafting" } else { name };
    Some((slug(name), display.to_string(), level))
}

struct Parsed {
    primary: Option<Station>,
    alts: Vec<Station>,
}

fn parse_stations(s: &str, names: &mut BTreeMap<String, String>) -> Parsed {
    let mut stations: Vec<Station> = Vec::new();
    for part in s.split(',') {
        if let Some((id, display, level)) = parse_station(part) {
            names.insert(id.clone(), display);
            stations.push(Station { workbench_id: id, level });
        }
    }
    let specialist = stations
        .iter()
        .position(|s| s.workbench_id != "workbench" && s.workbench_id != "inventory")
        .unwrap_or(0);
    let primary = (!stations.is_empty()).then(|| stations.remove(specialist));
    Parsed { primary, alts: stations }
}

fn unique_id(base: String, used: &mut BTreeSet<String>) -> String {
    let mut id = base.clone();
    let mut n = 2;
    while !used.insert(id.clone()) {
        id = format!("{base}-{n}");
        n += 1;
    }
    id
}

fn parse_modifier(entry: &str) -> Option<(Option<Modifier>, i32)> {
    let e = entry.trim();
    if e.is_empty() {
        return None;
    }
    if let Some(rest) = e.strip_prefix('+') {
        let (n, what) = rest.split_once(' ')?;
        return (what == "Magazine Size").then(|| (None, n.parse().unwrap_or(0)));
    }
    let (pct, rest) = e.split_once("% ")?;
    let pct: f64 = pct.parse().ok()?;
    let (dir, stat) = rest.split_once(' ')?;
    let sign = match dir {
        "Reduced" => -1.0,
        "Increased" => 1.0,
        _ => return None,
    };
    let stat = match stat.trim_end_matches(" Control") {
        "ADS Speed" => ModStat::AdsSpeed,
        "Base Dispersion" => ModStat::BaseDispersion,
        "Bullet Velocity" => ModStat::BulletVelocity,
        "Dispersion Recovery Time" => ModStat::DispersionRecovery,
        "Durability Burn Rate" => ModStat::DurabilityBurnRate,
        "Equip Time" => ModStat::EquipTime,
        "Distance Until Damage Falloff" => ModStat::FalloffDistance,
        "Fire Rate" => ModStat::FireRate,
        "Horizontal Recoil" => ModStat::HorizontalRecoil,
        "Max Shot Dispersion" => ModStat::MaxShotDispersion,
        "Noise" => ModStat::Noise,
        "Per-Shot Dispersion" => ModStat::PerShotDispersion,
        "Recoil Recovery Duration" | "Recoil Recovery Time" => ModStat::RecoilRecovery,
        "Unequip Time" => ModStat::UnequipTime,
        "Vertical Recoil" => ModStat::VerticalRecoil,
        _ => return None,
    };
    Some((Some(Modifier { stat, pct: sign * pct }), 0))
}

pub struct Output {
    pub files: DatasetFiles,
    pub unresolved: Vec<String>,
}

/// MetaForge lists each level's modifiers as `increasedFireRate` style fields, mostly zero.
fn weapon_upgrades(block: &Value) -> Vec<WeaponUpgrade> {
    let mut rows: Vec<WeaponUpgrade> = block.as_object().into_iter().flatten()
        .filter(|(key, _)| key.starts_with("increased") || key.starts_with("reduced"))
        .filter_map(|(key, value)| {
            let pct = value.as_f64().filter(|p| *p != 0.0)?;
            let (verb, rest) = key.split_at(if key.starts_with("reduced") { 7 } else { 9 });
            let words = rest.chars().fold(String::new(), |mut text, c| {
                if c.is_uppercase() && !text.is_empty() && !text.ends_with(|p: char| p.is_uppercase()) {
                    text.push(' ');
                }
                text.push(c);
                text
            });
            Some(WeaponUpgrade { label: format!("{} {words}", title_case(verb)), pct })
        })
        .collect();
    rows.sort_by(|a, b| a.label.cmp(&b.label));
    rows
}

pub fn normalize(src: &Sources, prec: &Precedence, overlay: &crate::skills::Overlay) -> Result<Output> {
    let mf = &src.metaforge;
    let wiki = &src.wiki;
    let rt_files = src.raidtheory["files"].as_object().context("raidtheory: no files")?;

    // Canonical ids come from names; MetaForge ids carry typos and duplicates.
    let mut mf_items: Vec<&Value> = rows(mf, "items").iter().collect();
    mf_items.sort_by_key(|i| text(i, "id"));
    let mut slug_uses: HashMap<String, u32> = HashMap::new();
    for i in &mf_items {
        *slug_uses.entry(slug(&text(i, "name"))).or_default() += 1;
    }

    let mut res = Resolver { by_name: HashMap::new(), by_mf_id: HashMap::new(), unresolved: BTreeSet::new() };
    let mut used_ids = BTreeSet::new();
    let mut canon: Vec<(String, &Value)> = Vec::new();
    for i in &mf_items {
        let name = text(i, "name");
        let s = slug(&name);
        let base = if slug_uses[&s] > 1 { slug(&text(i, "id")) } else { s };
        let id = unique_id(base, &mut used_ids);
        res.by_mf_id.insert(text(i, "id"), id.clone());
        res.insert(&name, &id);
        canon.push((id, i));
    }

    let wiki_items: HashMap<String, &Value> =
        wiki_rows(wiki, "items").map(|r| (decode(&text(r, "name")).to_lowercase(), r)).collect();
    let mut wiki_only: Vec<(String, &Value)> = Vec::new();
    for r in wiki_rows(wiki, "items") {
        let name = text(r, "name");
        if res.find(&name).is_none() && !name.is_empty() {
            // Sources differ on punctuation: Trigger 'Nade Blueprint is Trigger Nade Blueprint.
            let same = slug(&name);
            if canon.iter().any(|(id, _)| *id == same) {
                res.insert(&name, &same);
                continue;
            }
            let id = unique_id(slug(&name), &mut used_ids);
            res.insert(&name, &id);
            wiki_only.push((id, r));
        }
    }

    // Workbench names accumulate as recipes and upgrades are read.
    let mut bench_names = BTreeMap::new();

    // Recipes.
    let mut recipes: Vec<Recipe> = Vec::new();
    let mut recipe_ids = BTreeSet::new();
    let mut seen_craft: BTreeSet<String> = BTreeSet::new();
    let mut crafted: BTreeSet<String> = BTreeSet::new();
    let mut craft_rows: Vec<&Value> = wiki_rows(wiki, "crafting").collect();
    craft_rows.sort_by_key(|r| count(r, "row_id"));
    for r in craft_rows {
        let result = text(r, "resultItem");
        let Some(item_id) = res.name("craft output", &result) else { continue };
        let Some(inputs) = parse_ingredients(&mut res, &format!("craft input of {result}"), &text(r, "ingredients")) else {
            continue;
        };
        let station = text(r, "requireStation");
        if !seen_craft.insert(format!("{item_id}|{inputs:?}|{station}")) {
            continue;
        }
        crafted.insert(item_id.clone());
        let parsed = parse_stations(&station, &mut bench_names);
        let blueprint = text(r, "requireBlueprint");
        let blueprint_item_id =
            if blueprint.is_empty() { None } else { res.name("craft blueprint", &blueprint) };
        recipes.push(Recipe {
            id: unique_id(format!("{item_id}-craft"), &mut recipe_ids),
            kind: RecipeKind::Craft,
            output: Some(ItemQty { item_id, qty: count(r, "resultQty").unwrap_or(1) }),
            outputs: Vec::new(),
            inputs,
            workbench_id: parsed.primary.as_ref().map(|s| s.workbench_id.clone()),
            workbench_level: parsed.primary.as_ref().and_then(|s| s.level),
            alt_stations: parsed.alts,
            requires_blueprint: !blueprint.is_empty(),
            blueprint_item_id,
            required_skill: opt_text(r, "requireSkill"),
            craft_time_s: None,
            source_refs: SourceRefs { wiki: opt_text(r, "page"), ..Default::default() },
            provenance: Provenance::Wiki,
        });
    }

    let mut breakdown: BTreeMap<(String, &str), (Vec<ItemQty>, String)> = BTreeMap::new();
    let mut recycle_rows: Vec<&Value> = wiki_rows(wiki, "recycling").collect();
    recycle_rows.sort_by_key(|r| count(r, "row_id"));
    for r in recycle_rows {
        let kind = if text(r, "recipeType") == "salvaging" { "salvage" } else { "recycle" };
        let input = text(r, "inputItem");
        let Some(item_id) = res.name("recycle input", &input) else { continue };
        let Some(output) = res.name(&format!("recycle output of {input}"), &text(r, "outputItem")) else {
            continue;
        };
        let entry = breakdown.entry((item_id, kind)).or_insert_with(|| (Vec::new(), text(r, "page")));
        if !entry.0.iter().any(|q| q.item_id == output) {
            entry.0.push(ItemQty { item_id: output, qty: count(r, "outputQty").unwrap_or(1) });
        }
    }
    for ((item_id, kind), (outputs, page)) in breakdown {
        recipes.push(breakdown_recipe(&item_id, kind, outputs, opt_text_str(page), None, &mut recipe_ids, Provenance::Wiki));
    }

    // RaidTheory fills gaps only: it stopped updating in August.
    if prec.order("recipes").iter().any(|s| s == "raidtheory") {
        add_raidtheory_recipes(rt_files, &mut res, &mut recipes, &mut recipe_ids, &crafted, &mut bench_names);
    }

    // Weapon mods need weapon families; both need the raw item table.
    let mut family_ammo: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (_, i) in &canon {
        if text(i, "item_type") == "Weapon" {
            let name = text(i, "name");
            if let (Some(ammo), false) = (opt_text(i, "ammo_type"), name.starts_with("Amplified ")) {
                family_ammo.entry(weapon_family(&name)).or_default().insert(ammo.to_lowercase());
            }
        }
    }
    let mut weapon_mods = Vec::new();
    let mut slots: BTreeMap<String, BTreeMap<String, u8>> = BTreeMap::new();
    let mut mod_rows: Vec<&Value> = wiki_rows(wiki, "weaponMods").collect();
    mod_rows.sort_by_key(|r| text(r, "name"));
    for r in mod_rows {
        let name = text(r, "name");
        let Some(id) = res.name("weapon mod", &name) else { continue };
        let (family, tier) = split_tier(&name);
        let tier = tier.unwrap_or(1);
        let slot = slug(&text(r, "slot"));
        let mut modifiers = Vec::new();
        let mut magazine = 0;
        for entry in text(r, "function").split('@') {
            match parse_modifier(entry) {
                Some((m, flat)) => { modifiers.extend(m); magazine += flat; }
                None if !entry.trim().is_empty() => {
                    res.unresolved.insert(format!("mod modifier on {name}: {}", entry.trim()));
                }
                None => {}
            }
        }
        let weapons: Vec<String> =
            text(r, "compatible_weapons").split(',').map(slug).filter(|s| !s.is_empty()).collect();
        let mut ammo = BTreeSet::new();
        for w in &weapons {
            ammo.extend(family_ammo.get(w).into_iter().flatten().cloned());
            let max = slots.entry(w.clone()).or_default().entry(slot.clone()).or_insert(0);
            *max = (*max).max(tier);
        }
        let recipe = recipes.iter().find(|x| {
            x.kind == RecipeKind::Craft && x.output.as_ref().is_some_and(|o| o.item_id == id)
        });
        weapon_mods.push(WeaponMod {
            family: slug(family),
            tier,
            slot,
            compatible_ammo: ammo.into_iter().collect(),
            compatible_weapons: weapons,
            modifiers,
            flat: FlatModifiers { magazine_size: magazine },
            recipe_id: recipe.map(|x| x.id.clone()),
            gunsmith_level: recipe.and_then(|x| x.workbench_level),
            provenance: Provenance::Wiki,
            name,
            id,
        });
    }

    // Traders. MetaForge has the price, RaidTheory what it is priced in.
    let trades = trade_terms(rt_files, &mut res);
    let mut traders = Vec::new();
    let mut sold_by: BTreeMap<String, Vec<SoldBy>> = BTreeMap::new();
    for (name, stock) in mf["traders"].as_object().context("no traders")? {
        let trader_id = slug(name);
        let mut inventory = Vec::new();
        for row in stock.as_array().map(Vec::as_slice).unwrap_or(&[]) {
            let Some(item_id) = res.mf("trader stock", &text(row, "id")) else { continue };
            let price = count(row, "trader_price").unwrap_or(0);
            let terms = trades.items.get(&(trader_id.clone(), item_id.clone()));
            let currency = terms.map(|t| t.0.clone()).or_else(|| trades.usual.get(&trader_id).cloned());
            let daily_limit = terms.and_then(|t| t.1);
            sold_by.entry(item_id.clone()).or_default().push(SoldBy {
                trader: trader_id.clone(), price, currency: currency.clone(), daily_limit,
            });
            inventory.push(TraderStock { item_id, price, currency, daily_limit });
        }
        inventory.sort_by(|a, b| a.item_id.cmp(&b.item_id));
        traders.push(Trader { id: trader_id, name: name.clone(), inventory });
    }
    traders.sort_by(|a, b| a.id.cmp(&b.id));

    // Arcs, with drops from RaidTheory where it has them.
    let mut arcs: Vec<Arc> = rows(mf, "arcs").iter().map(|a| Arc {
        id: slug(&text(a, "id")),
        name: text(a, "name"),
        class: None,
        hp: None,
        icon: opt_text(a, "icon"),
        armor: None,
        attack: None,
        ability: None,
        weak_points: Vec::new(),
        tips: None,
        armor_zones: Vec::new(),
        drops: Vec::new(),
        source_refs: SourceRefs { metaforge: opt_text(a, "id"), ..Default::default() },
        provenance: Provenance::Api,
    }).collect();
    arcs.sort_by(|a, b| a.id.cmp(&b.id));
    let mut drops_from: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let rt_names = raidtheory_names(rt_files);
    for bot in rt_files.get("bots.json").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
        let arc_slug = slug(&text(bot, "name"));
        let Some(arc) = arcs.iter_mut().find(|a| a.id == arc_slug || slug(&a.name) == arc_slug) else {
            continue;
        };
        arc.class = opt_text(bot, "type");
        for rt_id in rows(bot, "drops").iter().filter_map(Value::as_str) {
            let Some(name) = rt_names.get(rt_id) else { continue };
            let Some(item_id) = res.name("arc drop", name) else { continue };
            if !arc.drops.iter().any(|d| d.item_id == item_id) {
                drops_from.entry(item_id.clone()).or_default().push(arc.id.clone());
                arc.drops.push(ArcDrop { item_id, chance: None });
            }
        }
    }

    // Items.
    let tiered: BTreeMap<String, (u8, String)> = canon.iter()
        .filter(|(_, i)| text(i, "item_type") == "Weapon" && !text(i, "name").starts_with("Amplified"))
        .fold(BTreeMap::new(), |mut best, (id, i)| {
            let name = text(i, "name");
            let tier = split_tier(&name).1.unwrap_or(1);
            let e = best.entry(weapon_family(&name)).or_insert((0, id.clone()));
            if tier > e.0 {
                *e = (tier, id.clone());
            }
            best
        });
    let rarity_of: HashMap<&str, Option<Rarity>> = canon.iter()
        .map(|(id, i)| (id.as_str(), parse_rarity(&text(i, "rarity"))))
        .collect();

    let mut items = Vec::new();
    for (id, i) in &canon {
        let name = text(i, "name");
        let block = &i["stat_block"];
        let wiki_row = wiki_items.get(&name.to_lowercase()).copied();
        let cat = category(&text(i, "item_type"), &name, block, !i["cosmetic"].is_null());

        let amplified = cat == Category::Weapon && name.starts_with("Amplified ");
        let family_name = name.trim_start_matches("Amplified ").split(" (").next().unwrap_or("");
        let base = amplified.then(|| tiered.get(&weapon_family(family_name))).flatten();
        let own_tier = split_tier(&name).1;
        let tier = match (amplified, base) {
            (true, Some((t, _))) => Some(*t),
            (true, None) => None,
            _ => match cat {
                Category::Weapon | Category::WeaponMod | Category::Shield => {
                    own_tier.or((cat == Category::Weapon).then_some(1))
                }
                Category::Augment => mk_tier(&name),
                _ => None,
            },
        };
        let rarity = match base {
            Some((_, base_id)) => rarity_of.get(base_id.as_str()).copied().flatten(),
            None => prec.pick("rarity", &[
                ("metaforge", parse_rarity(&text(i, "rarity"))),
                ("wiki", wiki_row.and_then(|r| parse_rarity(&text(r, "rarity")))),
            ]),
        };
        let shield_type = opt_text(i, "shield_type").or_else(|| {
            ["Light", "Medium", "Heavy"].iter().find(|t| name.contains(*t)).map(|t| t.to_lowercase())
        });
        let family_slots = slots.get(&weapon_family(if amplified { family_name } else { &name }));
        let mod_slots = family_slots.map(|m| {
            m.iter().map(|(slot, tier_max)| ModSlot { slot: slot.clone(), tier_max: *tier_max }).collect()
        }).unwrap_or_default();

        let mf_areas = opt_text(i, "loot_area").map(|s| split_areas(&s));
        let wiki_areas = wiki_row.and_then(|r| opt_text(r, "location")).map(|s| split_areas(&s));
        let loot_areas = prec
            .pick("loot_areas", &[("metaforge", mf_areas), ("wiki", wiki_areas)])
            .unwrap_or_default();

        items.push(Item {
            id: id.clone(),
            name: name.clone(),
            category: cat,
            subcategory: opt_text(i, "subcategory").filter(|s| s != &text(i, "item_type")),
            rarity,
            tier,
            description: prec.pick("description", &[
                ("metaforge", opt_text(i, "description")),
                ("wiki", wiki_row.and_then(|r| opt_text(r, "description"))),
            ]).unwrap_or_default(),
            flavor_text: text(i, "flavor_text"),
            icon: opt_text(i, "icon"),
            value: prec.pick("value", &[
                ("metaforge", count(i, "value")),
                ("wiki", wiki_row.and_then(|r| count(r, "sellprice"))),
            ]),
            weight_kg: prec.pick("weight_kg", &[
                ("metaforge", num(block, "weight")),
                ("wiki", wiki_row.and_then(|r| num(r, "weight"))),
            ]),
            stack_size: prec.pick("stack_size", &[
                ("metaforge", count(block, "stackSize")),
                ("wiki", wiki_row.and_then(|r| count(r, "stacksize"))),
            ]),
            stats: project_stats(cat, block, shield_type, mod_slots),
            ammo_type: opt_text(i, "ammo_type").map(|s| s.to_lowercase()),
            loadout_slots: rows(i, "loadout_slots").iter().filter_map(Value::as_str).map(slug).collect(),
            loot_areas,
            drops_from: drops_from.get(id).cloned().unwrap_or_default(),
            sold_by: sold_by.get(id).cloned().unwrap_or_default(),
            recipe_ids: Vec::new(),
            recycle_ids: Vec::new(),
            blueprint_required: false,
            variant_of: base.map(|(_, b)| b.clone()),
            stencil: None,
            source_refs: SourceRefs {
                metaforge: opt_text(i, "id"),
                wiki: wiki_row.and_then(|r| opt_text(r, "page")),
                raidtheory: None,
            },
            provenance: Provenance::Api,
        });
    }
    for (id, r) in &wiki_only {
        let name = text(r, "name");
        let cat = category(&text(r, "type"), &name, &Value::Null, false);
        items.push(Item {
            id: id.clone(),
            name: name.clone(),
            category: cat,
            subcategory: None,
            rarity: parse_rarity(&text(r, "rarity")),
            tier: split_tier(&name).1,
            description: text(r, "description"),
            flavor_text: String::new(),
            icon: None,
            value: count(r, "sellprice"),
            weight_kg: num(r, "weight"),
            stack_size: count(r, "stacksize"),
            stats: None,
            ammo_type: None,
            loadout_slots: Vec::new(),
            loot_areas: split_areas(&text(r, "location")),
            drops_from: drops_from.get(id).cloned().unwrap_or_default(),
            sold_by: sold_by.get(id).cloned().unwrap_or_default(),
            recipe_ids: Vec::new(),
            recycle_ids: Vec::new(),
            blueprint_required: false,
            variant_of: None,
            stencil: None,
            source_refs: SourceRefs { wiki: opt_text(r, "page"), ..Default::default() },
            provenance: Provenance::Wiki,
        });
    }
    items.sort_by(|a, b| a.id.cmp(&b.id));
    let item_pos: HashMap<String, usize> =
        items.iter().enumerate().map(|(n, i)| (i.id.clone(), n)).collect();
    recipes.sort_by(|a, b| a.id.cmp(&b.id));
    for r in &recipes {
        match r.kind {
            RecipeKind::Craft | RecipeKind::Trade => {
                if let Some(pos) = r.output.as_ref().and_then(|o| item_pos.get(&o.item_id)) {
                    items[*pos].recipe_ids.push(r.id.clone());
                    items[*pos].blueprint_required |= r.requires_blueprint;
                }
            }
            _ => {
                if let Some(pos) = r.inputs.first().and_then(|q| item_pos.get(&q.item_id)) {
                    items[*pos].recycle_ids.push(r.id.clone());
                }
            }
        }
    }
    for r in &recipes {
        if let Some(q) = r.output.as_ref() {
            if q.qty == 0 {
                res.unresolved.insert(format!("recipe {} yields 0", r.id));
            }
        }
    }

    // Workbenches.
    let mut upgrades: BTreeMap<String, BTreeMap<u8, Vec<ItemQty>>> = BTreeMap::new();
    let mut upgrade_rows: Vec<&Value> = wiki_rows(wiki, "workshopUpgrades").collect();
    upgrade_rows.sort_by_key(|r| count(r, "row_id"));
    for r in upgrade_rows {
        let station = text(r, "station");
        let Some(level) = count(r, "stationLevel") else { continue };
        let id = slug(&station);
        bench_names.entry(id.clone()).or_insert(station.clone());
        let Some(item_id) = res.name(&format!("{station} upgrade"), &text(r, "itemName")) else { continue };
        let list = upgrades.entry(id).or_default().entry(level as u8).or_default();
        merge_qty(list, item_id, qty(r));
    }
    let workbenches = bench_names.iter().map(|(id, name)| {
        let upgrade = upgrades.get(id);
        let recipe_max = recipes.iter()
            .flat_map(|r| r.workbench_id.as_deref().map(|w| (w, r.workbench_level)).into_iter()
                .chain(r.alt_stations.iter().map(|s| (s.workbench_id.as_str(), s.level))))
            .filter(|(w, _)| w == id)
            .filter_map(|(_, l)| l)
            .max()
            .unwrap_or(0);
        let upgrade_max = upgrade.and_then(|u| u.keys().max().copied()).unwrap_or(0);
        let max_level = recipe_max.max(upgrade_max);
        let levels = (1..=max_level).map(|level| WorkbenchLevel {
            level,
            upgrade_inputs: upgrade.and_then(|u| u.get(&level)).cloned().unwrap_or_default(),
            unlocks_recipe_ids: recipes.iter()
                .filter(|r| {
                    (r.workbench_id.as_deref() == Some(id) && r.workbench_level == Some(level))
                        || r.alt_stations.iter().any(|s| s.workbench_id == *id && s.level == Some(level))
                })
                .map(|r| r.id.clone())
                .collect(),
        }).collect();
        Workbench { id: id.clone(), name: name.clone(), max_level, levels }
    }).collect();

    // Maps and POIs.
    let mut pois = Vec::new();
    let mut maps = Vec::new();
    let events = rows(&mf["events"], "data");
    for (map_id, map_name) in MAPS {
        let markers = rows(&mf["maps"][*map_id], "markers");
        let mut poi_ids = Vec::new();
        let (mut xs, mut ys) = (Vec::new(), Vec::new());
        for m in markers {
            let (Some(x), Some(y)) = (num(m, "lng"), num(m, "lat")) else { continue };
            xs.push(x);
            ys.push(y);
            let sub = text(m, "subcategory");
            let loot_areas = split_areas(&text(m, "lootAreas"));
            let quest_id = (text(m, "category") == "quests").then(|| sub.clone());
            let kind = if quest_id.is_some() {
                PoiKind::QuestTarget
            } else if !loot_areas.is_empty() {
                PoiKind::Cache
            } else if ["extract", "exit", "elevator", "hatch", "metro"].iter().any(|k| sub.contains(k)) {
                PoiKind::Extract
            } else {
                PoiKind::Landmark
            };
            let uuid: String = text(m, "id").chars().filter(|c| *c != '-').take(12).collect();
            let id = format!("{map_id}-{uuid}");
            poi_ids.push(id.clone());
            pois.push(Poi {
                id,
                map_id: (*map_id).to_string(),
                name: opt_text(m, "instanceName").unwrap_or_else(|| title_case(&sub)),
                kind,
                x,
                y,
                loot_areas,
                notes: None,
                zlayers: m["zlayers"].as_u64().and_then(|z| u32::try_from(z).ok()),
                quest_id,
            });
        }
        let bounds = (!xs.is_empty()).then(|| Bounds {
            x_min: xs.iter().copied().fold(f64::MAX, f64::min),
            x_max: xs.iter().copied().fold(f64::MIN, f64::max),
            y_min: ys.iter().copied().fold(f64::MAX, f64::min),
            y_max: ys.iter().copied().fold(f64::MIN, f64::max),
        });
        let plain = map_name.trim_start_matches("The ");
        let event_pool: BTreeSet<String> = events.iter()
            .filter(|e| { let m = text(e, "map"); !m.is_empty() && plain.contains(m.as_str()) })
            .map(|e| text(e, "name"))
            .collect();
        poi_ids.sort();
        maps.push(GameMap {
            id: (*map_id).to_string(),
            name: (*map_name).to_string(),
            image: MapImage {
                url: None, width_px: None, height_px: None, tile_size: None,
                max_native_zoom: None, world_extent: None, tile_extent: None,
                layers: Vec::new(),
            },
            bounds,
            poi_ids,
            event_pool: event_pool.into_iter().collect(),
        });
    }
    pois.sort_by(|a, b| a.id.cmp(&b.id));
    pois.dedup_by(|a, b| a.id == b.id);
    for m in &mut maps {
        m.poi_ids.dedup();
    }

    // Loot areas: the union of what items and POIs mention.
    let mut area_items: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for i in &items {
        for a in &i.loot_areas {
            area_items.entry(a.clone()).or_default().push(i.id.clone());
        }
    }
    for p in &pois {
        for a in &p.loot_areas {
            area_items.entry(a.clone()).or_default();
        }
    }
    let loot_areas = area_items.into_iter().map(|(id, item_ids)| LootArea {
        name: title_case(&id),
        id,
        item_ids,
    }).collect();

    let quests = build_quests(mf, wiki, &mut res);
    let projects = build_projects(wiki, &mut res);

    let skills = crate::skills::build(&src.skills_wikitext, overlay)?;

    let version = DatasetVersion {
        dataset_version: src.date.clone(),
        game_patch: None,
        ingested_from: BTreeMap::from([
            ("metaforge".to_string(), src.date.clone()),
            ("wiki".to_string(), src.date.clone()),
            ("raidtheory".to_string(), text(&src.raidtheory, "commit")),
        ]),
    };

    let files = DatasetFiles {
        items, recipes, workbenches, weapon_mods, maps, pois, quests, projects, skills, arcs,
        traders, loot_areas, version,
    };
    Ok(Output { files, unresolved: res.unresolved.into_iter().collect() })
}

fn opt_text_str(s: String) -> Option<String> {
    Some(s).filter(|x| !x.is_empty())
}

fn title_case(s: &str) -> String {
    s.split(['-', '_', ' '])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn breakdown_recipe(
    item_id: &str,
    kind: &str,
    outputs: Vec<ItemQty>,
    wiki_page: Option<String>,
    rt_id: Option<String>,
    ids: &mut BTreeSet<String>,
    provenance: Provenance,
) -> Recipe {
    Recipe {
        id: unique_id(format!("{item_id}-{kind}"), ids),
        kind: if kind == "salvage" { RecipeKind::Salvage } else { RecipeKind::Recycle },
        output: None,
        outputs,
        inputs: vec![ItemQty { item_id: item_id.to_string(), qty: 1 }],
        workbench_id: None,
        workbench_level: None,
        alt_stations: Vec::new(),
        requires_blueprint: false,
        blueprint_item_id: None,
        required_skill: None,
        craft_time_s: None,
        source_refs: SourceRefs { wiki: wiki_page, raidtheory: rt_id, metaforge: None },
        provenance,
    }
}

/// `craftBench` is a bench id or a list of them, in RaidTheory's own naming.
fn raidtheory_stations(item: &Value) -> String {
    let benches: Vec<String> = match &item["craftBench"] {
        Value::String(s) => vec![s.clone()],
        Value::Array(a) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => Vec::new(),
    };
    let level = count(item, "stationLevelRequired");
    let named: Vec<String> = benches.iter().map(|b| {
        let name = match b.as_str() {
            "in_raid" => "Inventory".to_string(),
            "med_station" => "Medical Lab".to_string(),
            "weapon_bench" => "Gunsmith".to_string(),
            "equipment_bench" => "Gear Bench".to_string(),
            "explosives_bench" => "Explosives Station".to_string(),
            "utility_bench" => "Utility Station".to_string(),
            other => title_case(other),
        };
        match level {
            Some(l) if name != "Inventory" => format!("{name} {l}"),
            _ => name,
        }
    }).collect();
    named.join(",")
}

struct TradeTerms {
    /// (trader, item) -> (currency, daily limit)
    items: HashMap<(String, String), (String, Option<u32>)>,
    /// trader -> the currency most of its stock uses
    usual: HashMap<String, String>,
}

/// Trader ids differ across sources ("Tian Wen" / "TianWen"), hence the dash removal.
fn trade_terms(files: &serde_json::Map<String, Value>, res: &mut Resolver) -> TradeTerms {
    let names = raidtheory_names(files);
    let mut items = HashMap::new();
    let mut tally: BTreeMap<String, BTreeMap<String, u32>> = BTreeMap::new();
    for t in files.get("trades.json").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
        let trader = slug(&text(t, "trader")).replace('-', "");
        let currency = title_case(&text(&t["cost"], "itemId"));
        *tally.entry(trader.clone()).or_default().entry(currency.clone()).or_default() += 1;
        if let Some(id) = names.get(&text(t, "itemId")).and_then(|n| res.find(n)) {
            items.insert((trader, id), (currency, count(t, "dailyLimit")));
        }
    }
    let usual = tally.into_iter().filter_map(|(trader, counts)| {
        let best = counts.into_iter().max_by_key(|(_, n)| *n)?;
        Some((trader, best.0))
    }).collect();
    TradeTerms { items, usual }
}

fn raidtheory_names(files: &serde_json::Map<String, Value>) -> HashMap<String, String> {
    files.iter()
        .filter(|(k, _)| k.starts_with("items/"))
        .map(|(_, v)| (text(v, "id"), text(v, "name")))
        .collect()
}

fn add_raidtheory_recipes(
    files: &serde_json::Map<String, Value>,
    res: &mut Resolver,
    recipes: &mut Vec<Recipe>,
    ids: &mut BTreeSet<String>,
    crafted: &BTreeSet<String>,
    bench_names: &mut BTreeMap<String, String>,
) {
    let names = raidtheory_names(files);
    let broken_down: BTreeSet<String> = recipes.iter()
        .filter(|r| r.kind == RecipeKind::Recycle)
        .filter_map(|r| r.inputs.first().map(|q| q.item_id.clone()))
        .collect();
    let map_inputs = |res: &mut Resolver, ctx: &str, doc: &Value| -> Option<Vec<ItemQty>> {
        let mut out = Vec::new();
        for (rt_id, n) in doc.as_object()? {
            let id = names.get(rt_id).and_then(|name| res.name(ctx, name))?;
            merge_qty(&mut out, id, n.as_u64().unwrap_or(1) as u32);
        }
        (!out.is_empty()).then_some(out)
    };
    for (_, v) in files.iter().filter(|(k, _)| k.starts_with("items/")) {
        let rt_id = text(v, "id");
        let Some(item_id) = res.find(&text(v, "name")) else { continue };
        if !crafted.contains(&item_id) {
            if let Some(inputs) = map_inputs(res, "raidtheory recipe", &v["recipe"]) {
                let station = raidtheory_stations(v);
                let parsed = parse_stations(&station, bench_names);
                recipes.push(Recipe {
                    id: unique_id(format!("{item_id}-craft"), ids),
                    kind: RecipeKind::Craft,
                    output: Some(ItemQty {
                        item_id: item_id.clone(),
                        qty: count(v, "craftQuantity").unwrap_or(1),
                    }),
                    outputs: Vec::new(),
                    inputs,
                    workbench_id: parsed.primary.as_ref().map(|s| s.workbench_id.clone()),
                    workbench_level: parsed.primary.as_ref().and_then(|s| s.level),
                    alt_stations: parsed.alts,
                    requires_blueprint: false,
                    blueprint_item_id: None,
                    required_skill: None,
                    craft_time_s: None,
                    source_refs: SourceRefs { raidtheory: Some(rt_id.clone()), ..Default::default() },
                    provenance: Provenance::Community,
                });
            }
        }
        if !broken_down.contains(&item_id) {
            if let Some(outputs) = map_inputs(res, "raidtheory recycle", &v["recyclesInto"]) {
                recipes.push(breakdown_recipe(&item_id, "recycle", outputs, None, Some(rt_id), ids, Provenance::Community));
            }
        }
    }
}

fn build_quests(mf: &Value, wiki: &Value, res: &mut Resolver) -> Vec<Quest> {
    let wiki_quests: HashMap<String, &Value> =
        wiki_rows(wiki, "quests").map(|q| (slug(&text(q, "name")), q)).collect();
    let mut items_by_page: HashMap<String, Vec<&Value>> = HashMap::new();
    for r in wiki_rows(wiki, "questItems") {
        items_by_page.entry(slug(&text(r, "page"))).or_default().push(r);
    }
    let mut prereqs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for q in wiki_rows(wiki, "quests") {
        for next in text(q, "next").split(';').map(slug).filter(|s| !s.is_empty()) {
            prereqs.entry(next).or_default().insert(slug(&text(q, "name")));
        }
    }
    let known: BTreeSet<String> = rows(mf, "quests").iter().map(|q| slug(&text(q, "name"))).collect();

    let mut quests: Vec<Quest> = rows(mf, "quests").iter().map(|q| {
        let name = text(q, "name");
        let id = slug(&name);
        let w = wiki_quests.get(&id).copied();

        let api_maps: BTreeSet<String> = rows(q, "locations").iter().map(|l| text(l, "map")).collect();
        let wiki_map = w.and_then(|r| {
            let loc = slug(&text(r, "location"));
            MAPS.iter().find(|(m, n)| *m == loc || slug(n) == loc).map(|(m, _)| (*m).to_string())
        });
        let (maps, maps_provenance): (Vec<String>, Option<Provenance>) = if !api_maps.is_empty() {
            (api_maps.into_iter().collect(), Some(Provenance::Api))
        } else if let Some(m) = wiki_map {
            (vec![m], Some(Provenance::Wiki))
        } else {
            (Vec::new(), None)
        };

        let leaves: Vec<Objective> = rows(q, "objectives").iter().filter_map(Value::as_str)
            .enumerate()
            .map(|(n, s)| Objective {
                id: format!("o{}", n + 1),
                kind: "atomic".into(),
                children: Vec::new(),
                round_constraint: None,
                action: None,
                amount: None,
                description: Some(decode(s)),
                maps: maps.clone(),
                poi_ids: Vec::new(),
                item_ids: Vec::new(),
                maps_provenance,
            })
            .collect();
        let root = Objective {
            id: "o0".into(),
            kind: "sequence".into(),
            children: leaves.iter().map(|o| o.id.clone()).collect(),
            round_constraint: None,
            action: None,
            amount: None,
            description: None,
            maps: Vec::new(),
            poi_ids: Vec::new(),
            item_ids: Vec::new(),
            maps_provenance: None,
        };

        let page_rows = items_by_page.get(&id).map(Vec::as_slice).unwrap_or(&[]);
        let mut required_items = Vec::new();
        for r in rows(q, "required_items") {
            if let Some(item) = res.mf("quest requirement", &text(r, "item_id")) {
                merge_qty(&mut required_items, item, qty(r));
            }
        }
        if required_items.is_empty() {
            for r in page_rows.iter().filter(|r| text(r, "context") == "require") {
                if let Some(item) = res.name(&format!("quest {name} requirement"), &text(r, "itemName")) {
                    merge_qty(&mut required_items, item, qty(r));
                }
            }
        }
        let mut rewards = Vec::new();
        for r in rows(q, "rewards") {
            if let Some(item) = res.mf("quest reward", &text(r, "item_id")) {
                merge_qty(&mut rewards, item, qty(r));
            }
        }
        if rewards.is_empty() {
            for r in page_rows.iter().filter(|r| text(r, "context") == "reward") {
                if let Some(item) = res.name(&format!("quest {name} reward"), &text(r, "itemName")) {
                    merge_qty(&mut rewards, item, qty(r));
                }
            }
        }

        let prerequisites = prereqs.get(&id).into_iter().flatten()
            .filter(|p| known.contains(*p))
            .map(|p| QuestPrereq { quest_id: p.clone(), state: "completed".into() })
            .collect();

        Quest {
            trader: opt_text(q, "trader_name").map(|t| slug(&t)),
            cadence: "narrative".into(),
            prerequisites,
            objective_root: "o0".into(),
            objectives: std::iter::once(root).chain(leaves).collect(),
            required_items,
            rewards,
            xp: w.and_then(|r| count(r, "xp")).or_else(|| count(q, "xp")),
            graph_position: match (num(&q["position"], "x"), num(&q["position"], "y")) {
                (Some(x), Some(y)) => Some(GraphPosition { x, y }),
                _ => None,
            },
            provenance: Provenance::Api,
            name,
            id,
        }
    }).collect();
    quests.sort_by(|a, b| a.id.cmp(&b.id));
    quests.dedup_by(|a, b| a.id == b.id);
    quests
}

fn build_projects(wiki: &Value, res: &mut Resolver) -> Vec<Project> {
    let mut stages: BTreeMap<String, BTreeMap<(u32, u32), (Vec<ItemQty>, Vec<ItemQty>)>> = BTreeMap::new();
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    let mut item_rows: Vec<&Value> = wiki_rows(wiki, "projectItems").collect();
    item_rows.sort_by_key(|r| count(r, "row_id"));
    for r in item_rows {
        let page = text(r, "page");
        let id = slug(&page);
        names.entry(id.clone()).or_insert(page.clone());
        let key = (count(r, "projectPart").unwrap_or(1), count(r, "stage").unwrap_or(1));
        let entry = stages.entry(id).or_default().entry(key).or_default();
        let Some(item) = res.name(&format!("project {page}"), &text(r, "itemName")) else { continue };
        let list = if text(r, "context") == "reward" { &mut entry.1 } else { &mut entry.0 };
        merge_qty(list, item, qty(r));
    }
    let mut meta: HashMap<String, &Value> = HashMap::new();
    for p in wiki_rows(wiki, "projects") {
        let id = slug(&text(p, "name"));
        names.entry(id.clone()).or_insert(text(p, "name"));
        meta.insert(id, p);
    }
    names.into_iter().map(|(id, name)| {
        let parts = stages.remove(&id).unwrap_or_default();
        let multi_part = parts.keys().map(|k| k.0).collect::<BTreeSet<_>>().len() > 1;
        let stages = parts.into_iter().enumerate().map(|(n, ((part, stage), (inputs, rewards)))| ProjectStage {
            n: n as u32 + 1,
            title: if multi_part { format!("Part {part}, Stage {stage}") } else { format!("Stage {stage}") },
            inputs,
            rewards,
        }).collect();
        let deadline_days = meta.get(&id).and_then(|p| {
            let start = chrono::NaiveDate::parse_from_str(&text(p, "start"), "%Y-%m-%d").ok()?;
            let end = chrono::NaiveDate::parse_from_str(&text(p, "end"), "%Y-%m-%d").ok()?;
            u32::try_from((end - start).num_days()).ok()
        });
        let date = |key: &str| meta.get(&id).map(|p| text(p, key)).filter(|d| !d.is_empty());
        Project {
            expedition: name == "Expedition",
            starts: date("start"),
            ends: date("end"),
            permanent: false,
            paused: false,
            deadline_days,
            stages,
            id,
            name,
            provenance: Provenance::Wiki,
        }
    }).collect()
}
