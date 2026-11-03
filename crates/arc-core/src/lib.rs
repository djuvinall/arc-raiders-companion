//! Pure dataset core: load, index, query, calculators. No Tauri, no IO beyond data/.

pub mod advice;
pub mod bestiary;
pub mod calc;
pub mod dataset;
pub mod events;
pub mod model;
pub mod patches;
pub mod routing;
pub mod skills;
pub mod tracking;
pub mod views;
pub mod weapons;

pub use dataset::{Dataset, DatasetFiles, DATA_FILES};
