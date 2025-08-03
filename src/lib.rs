mod config;
mod score;
mod types;

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use config::ToricelliConfig;
use organic::parser::parse_file;
use organic::types::{Document, Heading, StandardProperties};

use sqlite::State;

fn fetch_notes(config: ToricelliConfig) {
    let query = "SELECT id, mtimes, stability, score FROM notes";
}

fn fetch_notes_from_org_roam_db(config: ToricelliConfig) {
    let connection = sqlite::open(config.org_roam_db).unwrap();
    let query = "SELECT * FROM \"main\".\"nodes\"";
    let mut statement = connection.prepare(query).unwrap();
    statement.bind((1, 50)).unwrap();

    while let Ok(State::Row) = statement.next() {
        println!("id = {}", statement.read::<String, _>("id").unwrap());
        println!("file = {}", statement.read::<i64, _>("file").unwrap());
    }
}
