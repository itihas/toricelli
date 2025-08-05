pub mod config;
pub mod score;
pub mod types;


use config::ToricelliConfig;

use sqlite::State;

pub fn fetch_notes(config: ToricelliConfig) {
    let query = "SELECT id, mtimes, stability, score FROM notes";
}

pub fn fetch_notes_from_org_roam_db(config: ToricelliConfig) {
    let connection = sqlite::open(config.org_roam_db).unwrap();
    let query = "SELECT * FROM \"main\".\"nodes\"";
    let mut statement = connection.prepare(query).unwrap();
    statement.bind((1, 50)).unwrap();

    while let Ok(State::Row) = statement.next() {
        println!("id = {}", statement.read::<String, _>("id").unwrap());
        println!("file = {}", statement.read::<i64, _>("file").unwrap());
    }
}
