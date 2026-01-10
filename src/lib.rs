#![feature(iterator_try_collect)]
pub mod config;
pub mod org_roam_db;
pub mod score;
pub mod types;

use std::{error::Error, fmt};

use config::ToricelliConfig;
use diesel::connection;
use sqlite::{Connection, Row, Value};
use types::{Note, NoteMap, ID};

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
    return Ok();
}

pub fn read_links(pool: &ConnectionPool, notemap: &mut NoteMap) -> Result<(), Box<dyn Error>> {
    let connection = &pool.main;
    let query = "SELECT * from links WHERE src=?;";
    let statement = connection.prepare(query)?;

    for (id,note) in notemap.iter_mut() {
	statement.bind((1, id.to_string()))?;
	
    };
}

pub fn create_resources(
    pool: &ConnectionPool,
    config: &ToricelliConfig,
) -> Result<(), Box<dyn Error>> {
    let query = "
         CREATE TABLE notes;
         CREATE TABLE links;
";
    let connection = &pool.main;
    connection.execute(query)?;
    Ok(())
}

pub fn read_note(
    id: ID,
    pool: &ConnectionPool,
    config: &ToricelliConfig,
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
    config: &ToricelliConfig,
) -> Result<(), Box<dyn Error>> {
    let connection = &pool.main;
    let query = "
UPDATE notes
SET mtimes=':mtimes',
    stability=':stability',
    score=':score',
    links=':links',
    backlinks=':backlinks',
    outlinks=':outlinks'
WHERE id=':id';
";

    // TODO make these blobs JSON blobs, and write or infer Serdes for them.
    let r = &[
        (":id", note.id.0.into()),
        (":mtimes", format!("{:?}", note.mtimes).into()),
        (":score", note.score.to_string().into()),
        (":stability", note.stability.to_string().into()),
        (":links", format!("{:?}", note.links).into()),
        (":backlinks", format!("{:?}", note.backlinks).into()),
        (":outlinks", format!("{:?}", note.outlinks).into()),
    ][..];
    let mut statement = connection.prepare(query)?;
    statement.bind::<&[(_, sqlite::Value)]>(r)?;
    Ok(())
}
