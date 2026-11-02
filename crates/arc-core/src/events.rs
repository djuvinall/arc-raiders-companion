//! Live map events (plan §4.10, §8). The schedule is fetched and cached by the shell;
//! this only reads it, so it stays pure and testable.

use crate::dataset::Dataset;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub map: String,
    pub icon: Option<String>,
    #[serde(rename = "startTime")]
    pub start_ms: i64,
    #[serde(rename = "endTime")]
    pub end_ms: i64,
}

#[derive(Deserialize)]
struct Schedule {
    data: Vec<Event>,
}

pub fn parse_schedule(json: &[u8]) -> Result<Vec<Event>, serde_json::Error> {
    Ok(serde_json::from_slice::<Schedule>(json)?.data)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapEvents {
    pub map_id: String,
    pub map_name: String,
    pub current: Vec<Event>,
    pub next: Option<Event>,
}

/// The feed says "Dam" where the dataset says "Dam Battlegrounds".
fn is_map(event_map: &str, map_name: &str) -> bool {
    map_name.to_lowercase().contains(&event_map.to_lowercase())
}

pub fn events_now(ds: &Dataset, schedule: &[Event], now_ms: i64) -> Vec<MapEvents> {
    ds.files.maps.iter().map(|map| {
        let here: Vec<&Event> = schedule.iter().filter(|e| is_map(&e.map, &map.name)).collect();
        MapEvents {
            map_id: map.id.clone(),
            map_name: map.name.clone(),
            current: here.iter().filter(|e| e.start_ms <= now_ms && now_ms < e.end_ms).map(|e| (*e).clone()).collect(),
            next: here.iter().filter(|e| e.start_ms > now_ms).min_by_key(|e| e.start_ms).map(|e| (*e).clone()),
        }
    }).collect()
}
