#![feature(iterator_try_collect)]
pub mod config;
pub mod org_file;
pub mod org_roam_db;
pub mod score;
pub mod types;

use std::collections::HashMap;
use std::error::Error;
use chrono::Utc;
use config::ToricelliConfig;
use org_file::update_file_properties;
use score::{pagerank, update_standalone_score};
use serde_json;
use sqlite::{Connection, Value};
use types::{LinkGraph, Note, NoteMap, NoteStore, ID};

pub struct ConnectionPool {
    pub main: Connection,
    pub org_roam: Connection,
}

/// Fetch all notes from the database into a NoteStore.
/// All notes start with clean (not dirty) state.
pub fn fetch_notes(pool: &ConnectionPool, store: &mut NoteStore) -> Result<(), Box<dyn Error>> {
    let connection = &pool.main;
    let query = "SELECT * from notes;";
    let statement = connection.prepare(query)?;

    let mut notes = NoteMap::new();
    for row in statement.into_iter().map(|row| row.unwrap()) {
        let values: Vec<Value> = row.into();
        let note: Note = values.into();
        notes.insert(note.id.clone(), note);
    }
    *store = NoteStore::from_notes(notes);
    Ok(())
}

pub fn fetch_link_graph(pool: &ConnectionPool) -> Result<LinkGraph, Box<dyn Error>> {
    let connection = &pool.main;
    let query = "SELECT src, dest, weight FROM links;";
    let statement = connection.prepare(query)?;

    let edges: Vec<(ID, ID, f64)> = statement
        .into_iter()
        .map(|row| {
            let row = row.unwrap();
            let src = ID::from(row.read::<&str, _>("src"));
            let dest = ID::from(row.read::<&str, _>("dest"));
            let weight = row.read::<f64, _>("weight");
            (src, dest, weight)
        })
        .collect();

    Ok(LinkGraph::from_edges(edges))
}

pub fn create_resources(
    pool: &ConnectionPool,
    _config: &ToricelliConfig,
) -> Result<(), Box<dyn Error>> {
    let query = "
         CREATE TABLE IF NOT EXISTS notes (id TEXT PRIMARY KEY, data TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS links (
             src TEXT NOT NULL,
             dest TEXT NOT NULL,
             weight REAL DEFAULT 1.0,
             PRIMARY KEY (src, dest)
         );
         CREATE INDEX IF NOT EXISTS idx_links_dest ON links(dest);
";
    println!("{:?}", query);
    let connection = &pool.main;
    connection.execute(query)?;
    println!("created");
    Ok(())
}

/// Flush only dirty notes from the store to the database.
/// Uses INSERT OR REPLACE for upsert semantics.
/// Clears dirty flags on success.
pub fn flush_notes(pool: &ConnectionPool, store: &mut NoteStore) -> Result<usize, Box<dyn Error>> {
    let connection = &pool.main;
    let query = "INSERT OR REPLACE INTO notes (id, data) VALUES (:id, :data);";

    let mut flushed = 0;
    connection.execute("BEGIN")?;
    let mut statement = connection.prepare(query)?;
    for note in store.dirty_notes() {
        let data = serde_json::to_string(&note)?;
        statement.bind::<&[(_, sqlite::Value)]>(
            &[(":id", note.id.0.clone().into()), (":data", data.into())][..],
        )?;
        statement.next()?;
        statement.reset()?;
        flushed += 1;
    }
    connection.execute("COMMIT")?;

    store.clear_dirty();
    Ok(flushed)
}

/// Flush only dirty links from the store to the database.
/// Uses INSERT OR REPLACE for upsert semantics.
/// Clears dirty flags on success.
pub fn flush_links(pool: &ConnectionPool, graph: &mut LinkGraph) -> Result<usize, Box<dyn Error>> {
    let connection = &pool.main;
    let query = "INSERT OR REPLACE INTO links (src, dest, weight) VALUES (:src, :dest, :weight);";

    let mut flushed = 0;
    connection.execute("BEGIN")?;
    let mut statement = connection.prepare(query)?;
    for src in graph.dirty_ids() {
        if let Some(dests) = graph.links(&src) {
            for (dest, weight) in dests {
                statement.bind(
                    &[
                        (":src", src.0.clone().as_str()),
                        (":dest", dest.0.clone().as_str()),
                        (":weight", weight.to_string().as_str()),
                    ][..],
                )?;
                statement.next()?;
                statement.reset()?;
            }
        }
        flushed += 1;
    }
    connection.execute("COMMIT")?;

    graph.clear_dirty();
    Ok(flushed)
}

/// Record a review of a note: push a new mtime, recompute standalone
/// score, then run PageRank to propagate the change through the graph.
/// All mutations go through NoteStore dirty tracking.
/// Returns Err if the note ID is not found.
pub fn review(id: &ID, store: &mut NoteStore, graph: &LinkGraph) -> Result<(), Box<dyn Error>> {
    if store.get(id).is_none() {
        return Err(format!("note not found: {}", id).into());
    }

    // 1. Record the review time
    store.update(id, |note| {
        note.mtimes.push(Utc::now());
    });

    // 2. Recompute standalone score for this note
    store.update(id, |note| {
        update_standalone_score(note);
    });

    // 3. Run PageRank to propagate score changes through the graph
    pagerank(store, graph);

    Ok(())
}

/// Result of flushing properties to files.
#[derive(Default)]
pub struct FlushPropertiesResult {
    pub updated: usize,
    pub skipped: usize,
    pub errors: Vec<(ID, String)>,
}

/// Flush property changes to org file frontmatter.
/// Only flushes properties listed in config.flush_properties.
/// If flush_properties is empty, flushes all Note fields.
/// Returns the number of files successfully updated.
pub fn flush_properties_to_files(
    store: &NoteStore,
    config: &ToricelliConfig,
) -> FlushPropertiesResult {
    let mut result = FlushPropertiesResult::default();

    for note in store.dirty_notes() {
        let Some(ref file_path) = note.file else {
            result.skipped += 1;
            continue;
        };

        // Build property updates based on config
        let updates = build_property_updates(note, &config.flush_properties);

        match update_file_properties(file_path, &updates) {
            Ok(()) => result.updated += 1,
            Err(e) => {
                result.errors.push((note.id.clone(), e.to_string()));
            }
        }
    }

    result
}

/// Build a HashMap of property updates from a Note based on configured properties.
fn build_property_updates(note: &Note, properties: &[String]) -> HashMap<String, String> {
    let mut updates = HashMap::new();

    // If properties list is empty, flush all Note fields
    let flush_all = properties.is_empty();

    for prop in properties.iter().map(|s| s.as_str()).chain(if flush_all {
        vec!["STABILITY", "SCORE", "MTIMES"].into_iter()
    } else {
        vec![].into_iter()
    }) {
        match prop {
            "STABILITY" => {
                updates.insert("STABILITY".to_string(), format!("{:.4}", note.stability));
            }
            "SCORE" => {
                updates.insert("SCORE".to_string(), format!("{:.4}", note.score));
            }
            "MTIMES" => {
                if !note.mtimes.is_empty() {
                    let mtimes_str = note
                        .mtimes
                        .iter()
                        .map(|dt| dt.format("[%Y-%m-%d %a %H:%M]").to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    updates.insert("MTIME".to_string(), mtimes_str);
                }
            }
            _ => {} // Unknown property, skip
        }
    }

    updates
}
