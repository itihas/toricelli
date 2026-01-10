#![feature(iterator_try_collect)]
pub mod config;
pub mod org_roam_db;
pub mod score;
pub mod types;

use std::error::Error;

use config::ToricelliConfig;
use sqlite::{Connection, Value};
use types::{Note, NoteMap, ID};
use serde_json;

pub struct ConnectionPool {
    pub main: Connection,
    pub org_roam: Connection,
}

pub fn fetch_notes(pool: &ConnectionPool, notemap: &mut NoteMap) -> Result<(), Box<dyn Error>> {
    let connection = &pool.main;
    let query = format!("SELECT * from notes;");
    let statement = connection.prepare(query)?;
    for row in statement.into_iter().map(|row| row.unwrap()) {
        let values: Vec<Value> = row.into();
	let note: Note = values.into();
        notemap.insert(note.id.clone(), note);
    }
    Ok(())
}

pub fn read_links(pool: &ConnectionPool, notemap: &mut NoteMap) -> Result<(), Box<dyn Error>> {
    let connection = &pool.main;
    let query = "SELECT * from links WHERE src=?;";

    for (id, _note) in notemap.iter_mut() {
        let mut statement = connection.prepare(query)?;
        statement.bind((1, id.0.as_str()))?;
        // TODO: iterate statement results and populate note.links
    }
    Ok(())
}

pub fn create_resources(
    pool: &ConnectionPool,
    _config: &ToricelliConfig,
) -> Result<(), Box<dyn Error>> {
    let query = "
         CREATE TABLE notes (id TEXT PRIMARY KEY, data TEXT NOT NULL);
         CREATE TABLE links (src TEXT, dest TEXT);
";
    let connection = &pool.main;
    connection.execute(query)?;
    Ok(())
}

pub fn read_note(
    id: ID,
    pool: &ConnectionPool,
    _config: &ToricelliConfig,
) -> Result<Note, Box<dyn Error>> {
    let connection = &pool.main;
    let query = format!("SELECT * from notes WHERE id={};", id.to_string());
    connection
        .prepare(query)?
        .into_iter()
        .try_next()?
        .map(|v| Ok(v.into()))
        .unwrap()
}

pub fn update_note(
    note: Note,
    pool: &ConnectionPool,
    _config: &ToricelliConfig,
) -> Result<(), Box<dyn Error>> {
    let connection = &pool.main;
    let query = "
UPDATE notes
SET data=:data
WHERE id=:id;
";

    let data = serde_json::to_string(&note)?;
    let r = &[
        (":id", note.id.0.into()),
        (":data", data.into()),
    ][..];
    let mut statement = connection.prepare(query)?;
    statement.bind::<&[(_, sqlite::Value)]>(r)?;
    Ok(())
}
