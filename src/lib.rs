#![feature(iterator_try_collect)]
pub mod config;
pub mod org_roam_db;
pub mod score;
pub mod types;

use config::ToricelliConfig;

pub fn fetch_notes(config: ToricelliConfig) {
    let query = "SELECT id, mtimes, stability, score FROM notes";
}
