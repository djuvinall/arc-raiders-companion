//! Phase 3 calculators (D9): stat sheet with mod deltas and side-by-side compare. All pure.
//!
//! Percent modifiers multiply the matching stat and collisions on one stat multiply
//! sequentially; flat values add. That is a documented guess: the game's real rule is
//! unknown. Most mod stats have no UI-stat counterpart, so they are listed as effects
//! rather than folded into a number the game doesn't show.

use crate::dataset::Dataset;
use crate::model::*;
use crate::views::{item_link, Link};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SlotOptions {
    pub slot: String,
    pub tier_max: u8,
    pub mods: Vec<WeaponMod>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatRow {
    pub key: String,
    pub label: String,
    pub base: Option<f64>,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Effect {
    pub stat: String,
    pub pct: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ttk {
    pub target_hp: f64,
    pub shots: u32,
    pub seconds: f64,
}

/// Shots and time to down a target wearing one kind of shield.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KillTime {
    pub shots: u32,
    /// Null without a measured fire rate.
    pub seconds: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ShieldTtk {
    /// `none`, or the shield's item id.
    pub shield: String,
    pub name: String,
    pub charge: f64,
    /// Percent of each hit the shield takes while it has charge.
    pub reduction_pct: f64,
    pub body: Option<KillTime>,
    /// Every shot to the head; null without a headshot multiplier.
    pub head: Option<KillTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sheet {
    pub weapon: Link,
    pub firing_mode: Option<String>,
    pub arc_armor_penetration: Option<String>,
    pub ammo_type: Option<String>,
    pub rows: Vec<StatRow>,
    pub upgrades: Vec<WeaponUpgrade>,
    pub effects: Vec<Effect>,
    pub measured: Vec<StatRow>,
    /// Null until `measured.rounds_per_minute` exists.
    pub ttk: Option<Ttk>,
    /// Damage per shot, rounds per minute (level and mods included) and headshot
    /// multiplier, for charts.
    pub damage: Option<f64>,
    pub rpm: Option<f64>,
    pub headshot_mult: Option<f64>,
    pub vs_shields: Vec<ShieldTtk>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompareRow {
    pub label: String,
    pub values: Vec<Option<f64>>,
    /// Column holding the highest value; null on a tie or when fewer than two have one.
    pub best: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompareTable {
    pub columns: Vec<String>,
    pub rows: Vec<CompareRow>,
}

pub(crate) fn weapon_stats(item: &Item) -> Option<&WeaponStats> {
    match &item.stats {
        Some(Stats::Weapon(w)) => Some(w),
        _ => None,
    }
}

/// `rattler-iv` and `aphelion-rifle` to the family slugs mods list.
pub fn weapon_family(ds: &Dataset, id: &str) -> String {
    let base = ds.item(id).and_then(|i| i.variant_of.as_deref()).unwrap_or(id);
    let family = ["-iv", "-iii", "-ii", "-i"].iter().find_map(|s| base.strip_suffix(s)).unwrap_or(base);
    family.trim_end_matches("-rifle").to_string()
}

/// `Anvil IV` and `Amplified Anvil (X-Rounds)` both read `Anvil`.
pub fn weapon_family_name(ds: &Dataset, id: &str) -> String {
    let item = ds.item(id);
    let base = item.and_then(|i| i.variant_of.as_deref()).and_then(|b| ds.item(b)).or(item);
    let name = base.map_or(id, |i| i.name.as_str());
    [" IV", " III", " II", " I"].iter().find_map(|s| name.strip_suffix(s)).unwrap_or(name).to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WeaponLevel {
    pub id: String,
    pub level: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WeaponVariant {
    pub id: String,
    pub amplifications: Vec<String>,
}

/// One gun: its levels and the amplified versions the data lists.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WeaponFamily {
    pub name: String,
    pub levels: Vec<WeaponLevel>,
    pub amplified: Vec<WeaponVariant>,
}

/// `Amplified Aphelion (Bigger Mag, Charged Burst)` to its two amplifications.
fn amplifications(name: &str) -> Vec<String> {
    name.split_once('(').map_or(Vec::new(), |(_, rest)| {
        rest.trim_end_matches(')').split(", ").map(str::to_string).collect()
    })
}

pub fn weapon_family_view(ds: &Dataset, id: &str) -> Option<WeaponFamily> {
    weapon_stats(ds.item(id)?)?;
    let family = weapon_family(ds, id);
    let members = || ds.files.items.iter()
        .filter(|i| weapon_stats(i).is_some() && weapon_family(ds, &i.id) == family);
    let mut levels: Vec<WeaponLevel> = members().filter(|i| i.variant_of.is_none())
        .map(|i| WeaponLevel { id: i.id.clone(), level: i.tier.unwrap_or(1) })
        .collect();
    levels.sort_by_key(|l| l.level);
    let amplified = members().filter(|i| i.variant_of.is_some())
        .map(|i| WeaponVariant { id: i.id.clone(), amplifications: amplifications(&i.name) })
        .collect();
    Some(WeaponFamily { name: weapon_family_name(ds, id), levels, amplified })
}

/// The guns a stencil fits, each shown by its lowest level.
pub fn stencil_guns(ds: &Dataset, stencil: &StencilFit) -> Vec<Link> {
    stencil.weapons.iter().filter_map(|family| {
        let lead = ds.files.items.iter()
            .filter(|i| weapon_stats(i).is_some() && i.variant_of.is_none() && weapon_family(ds, &i.id) == *family)
            .min_by_key(|i| i.tier)?;
        Some(Link { name: weapon_family_name(ds, &lead.id), ..item_link(ds, &lead.id, None) })
    }).collect()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GunStencil {
    pub stencil: Link,
    pub swatch: String,
}

/// The reverse of [`stencil_guns`]: what fits this gun, at any level or version.
pub fn gun_stencils(ds: &Dataset, id: &str) -> Vec<GunStencil> {
    let Some(_) = ds.item(id).and_then(weapon_stats) else { return Vec::new() };
    let family = weapon_family(ds, id);
    ds.files.items.iter().filter_map(|i| {
        let fit = i.stencil.as_ref().filter(|s| s.weapons.contains(&family))?;
        Some(GunStencil { stencil: item_link(ds, &i.id, None), swatch: fit.swatch.clone() })
    }).collect()
}

pub fn fits(ds: &Dataset, item_id: &str, slot: &ModSlot, m: &WeaponMod) -> bool {
    let family = weapon_family(ds, item_id);
    m.slot == slot.slot
        && m.tier <= slot.tier_max
        && (m.compatible_weapons.is_empty() || m.compatible_weapons.contains(&family))
}

pub fn slot_options(ds: &Dataset, item_id: &str) -> Option<Vec<SlotOptions>> {
    let stats = weapon_stats(ds.item(item_id)?)?;
    Some(stats.mod_slots.iter().map(|slot| SlotOptions {
        slot: slot.slot.clone(),
        tier_max: slot.tier_max,
        mods: ds.files.weapon_mods.iter().filter(|m| fits(ds, item_id, slot, m)).cloned().collect(),
    }).collect())
}

/// Combined multiplier for one stat: 10% then 10% is 1.21.
fn multiplier(mods: &[&WeaponMod], stat: ModStat) -> f64 {
    mods.iter()
        .flat_map(|m| &m.modifiers)
        .filter(|x| x.stat == stat)
        .fold(1.0, |total, x| total * (1.0 + x.pct / 100.0))
}

fn stat_name(stat: ModStat) -> String {
    serde_json::to_value(stat).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default()
}

const ALL_STATS: [ModStat; 15] = [
    ModStat::AdsSpeed, ModStat::BaseDispersion, ModStat::BulletVelocity, ModStat::DispersionRecovery,
    ModStat::DurabilityBurnRate, ModStat::EquipTime, ModStat::FalloffDistance, ModStat::FireRate,
    ModStat::HorizontalRecoil, ModStat::MaxShotDispersion, ModStat::Noise, ModStat::PerShotDispersion,
    ModStat::RecoilRecovery, ModStat::UnequipTime, ModStat::VerticalRecoil,
];

fn row(key: &str, label: &str, base: Option<f64>, value: Option<f64>) -> StatRow {
    StatRow { key: key.into(), label: label.into(), base, value }
}

/// Plan §6 Phase 3: ceil(hp / damage) shots, with `seconds` taken from the interval
/// between them. The plan's formula, so it counts the first shot as taking one interval.
pub fn ttk_flat(target_hp: f64, damage: f64, rounds_per_minute: f64) -> Option<Ttk> {
    if damage <= 0.0 || rounds_per_minute <= 0.0 || target_hp <= 0.0 {
        return None;
    }
    let shots = (target_hp / damage).ceil();
    Some(Ttk { target_hp, shots: shots as u32, seconds: shots * (60.0 / rounds_per_minute) })
}

/// Shots to take `hp` down through a shield. While the shield has charge it takes
/// `reduction` of each hit and health takes the rest; once it's empty, health takes all
/// of it. That's the community's reading of the game's shield text, not a published rule.
pub fn shots_to_kill(hp: f64, charge: f64, reduction: f64, damage: f64) -> Option<u32> {
    if damage <= 0.0 || hp <= 0.0 {
        return None;
    }
    let (mut hp, mut charge) = (hp, charge.max(0.0));
    for shot in 1..=10_000 {
        let blocked = (damage * reduction.clamp(0.0, 1.0)).min(charge);
        charge -= blocked;
        hp -= damage - blocked;
        if hp <= 1e-9 {
            return Some(shot);
        }
    }
    None
}

/// Kill times against no shield and each shield in the data, lightest first.
pub fn vs_shields(ds: &Dataset, hp: f64, damage: Option<f64>, rpm: Option<f64>, head: Option<f64>) -> Vec<ShieldTtk> {
    let mut shields: Vec<(String, String, f64, f64)> = vec![("none".into(), "No shield".into(), 0.0, 0.0)];
    let mut worn: Vec<(String, String, f64, f64)> = ds.files.items.iter().filter_map(|i| match &i.stats {
        Some(Stats::Shield(s)) => Some((i.id.clone(), i.name.clone(), s.charge?, s.damage_mitigation?)),
        _ => None,
    }).collect();
    worn.sort_by(|a, b| a.2.total_cmp(&b.2));
    shields.extend(worn);
    let time = |shots: u32| rpm.filter(|r| *r > 0.0).map(|r| f64::from(shots) * 60.0 / r);
    shields.into_iter().map(|(shield, name, charge, pct)| {
        let kill = |d: Option<f64>| d.and_then(|d| shots_to_kill(hp, charge, pct / 100.0, d))
            .map(|shots| KillTime { shots, seconds: time(shots) });
        ShieldTtk {
            body: kill(damage),
            head: kill(damage.zip(head).map(|(d, m)| d * m)),
            shield, name, charge, reduction_pct: pct,
        }
    }).collect()
}

pub fn apply_mods(ds: &Dataset, item_id: &str, mod_ids: &[String], target_hp: f64) -> Option<Sheet> {
    let item = ds.item(item_id)?;
    let stats = weapon_stats(item)?;
    let mut warnings = Vec::new();
    let mut used_slots: Vec<&str> = Vec::new();
    let mut mods = Vec::new();
    for id in mod_ids {
        let Some(m) = ds.files.weapon_mods.iter().find(|m| m.id == *id) else {
            warnings.push(format!("Unknown mod '{id}'."));
            continue;
        };
        if !stats.mod_slots.iter().any(|s| fits(ds, item_id, s, m)) {
            warnings.push(format!("{} doesn't fit {}.", m.name, item.name));
        } else if used_slots.contains(&m.slot.as_str()) {
            warnings.push(format!("Two mods in the {} slot; only the first counts.", m.slot));
        } else {
            used_slots.push(&m.slot);
            mods.push(m);
        }
    }

    // A level's fire rate bonus isn't in the listed stat, which is the same at every level.
    let bonus = stats.upgrades.iter().find(|u| u.label == "Increased Fire Rate").map_or(0.0, |u| u.pct);
    let base_rate = stats.fire_rate.map(|f| f * (1.0 + bonus / 100.0));
    let fire_rate = base_rate.map(|f| f * multiplier(&mods, ModStat::FireRate));
    let magazine_flat: i32 = mods.iter().map(|m| m.flat.magazine_size).sum();
    let base_mag = stats.magazine_size.map(f64::from);
    let dps = |rate: Option<f64>| stats.damage.zip(rate).map(|(d, r)| d * r);
    let rows = vec![
        row("damage", "Damage", stats.damage, stats.damage),
        row("fire_rate", "Fire rate", base_rate, fire_rate),
        row("stat_dps", "Stat DPS", dps(base_rate), dps(fire_rate)),
        row("range", "Range", stats.range, stats.range),
        row("stability", "Stability", stats.stability, stats.stability),
        row("agility", "Agility", stats.agility, stats.agility),
        row("stealth", "Stealth", stats.stealth, stats.stealth),
        row("magazine_size", "Magazine", base_mag, base_mag.map(|m| m + f64::from(magazine_flat))),
    ];

    let effects = ALL_STATS.iter().filter_map(|&stat| {
        let pct = (multiplier(&mods, stat) - 1.0) * 100.0;
        (pct.abs() > 1e-9).then(|| Effect { stat: stat_name(stat), pct })
    }).collect();

    let m = &stats.measured;
    let scale = |v: Option<f64>, stat| v.map(|x| x * multiplier(&mods, stat));
    // The wiki's RPM is the gun's base, so the level's fire rate bonus applies here too.
    let level_rpm = m.rounds_per_minute.map(|r| r * (1.0 + bonus / 100.0));
    let rpm = scale(level_rpm, ModStat::FireRate);
    let measured = vec![
        row("rounds_per_minute", "Rounds per minute", level_rpm, rpm),
        row("bullet_velocity_ms", "Bullet velocity m/s", m.bullet_velocity_ms, scale(m.bullet_velocity_ms, ModStat::BulletVelocity)),
        row("falloff_start_m", "Falloff start m", m.falloff_start_m, scale(m.falloff_start_m, ModStat::FalloffDistance)),
        row("falloff_end_m", "Falloff end m", m.falloff_end_m, scale(m.falloff_end_m, ModStat::FalloffDistance)),
    ].into_iter().filter(|r| r.base.is_some()).collect();

    Some(Sheet {
        weapon: item_link(ds, item_id, None),
        firing_mode: stats.firing_mode.clone(),
        arc_armor_penetration: stats.arc_armor_penetration.clone(),
        ammo_type: item.ammo_type.clone(),
        rows,
        upgrades: stats.upgrades.clone(),
        effects,
        measured,
        ttk: stats.damage.zip(rpm).and_then(|(d, r)| ttk_flat(target_hp, d, r)),
        damage: stats.damage,
        rpm,
        headshot_mult: m.headshot_mult,
        vs_shields: vs_shields(ds, target_hp, stats.damage, rpm, m.headshot_mult),
        warnings,
    })
}

fn best_of(values: &[Option<f64>]) -> Option<usize> {
    let known: Vec<(usize, f64)> = values.iter().enumerate().filter_map(|(i, v)| v.map(|x| (i, x))).collect();
    if known.len() < 2 {
        return None;
    }
    let top = known.iter().map(|(_, v)| *v).fold(f64::MIN, f64::max);
    let mut leaders = known.iter().filter(|(_, v)| (*v - top).abs() < 1e-9);
    let first = leaders.next()?.0;
    leaders.next().is_none().then_some(first)
}

pub fn compare(sheets: &[(String, Sheet)]) -> CompareTable {
    let columns = sheets.iter().map(|(name, _)| name.clone()).collect();
    let rows = sheets.first().map_or(Vec::new(), |(_, first)| {
        first.rows.iter().enumerate().map(|(i, r)| {
            let values: Vec<Option<f64>> = sheets.iter().map(|(_, s)| s.rows[i].value).collect();
            CompareRow { label: r.label.clone(), best: best_of(&values), values }
        }).collect()
    });
    CompareTable { columns, rows }
}
