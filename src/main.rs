use std::error::Error;

use toricelli::config::ToricelliConfig;
use toricelli::org_roam_db::fetch_from_org_roam_db;
use toricelli::{ConnectionPool, create_resources, flush_links, flush_notes};
use toricelli::types::{LinkGraph, NoteStore};

fn main() -> Result<(), Box<dyn Error>> {
    let config = ToricelliConfig::from_env();
    let pool = ConnectionPool {
        main: sqlite::open(&config.db)?,
        org_roam: sqlite::open(&config.org_roam_db)?,
    };
    create_resources(&pool, &config)?;
    let mut store = NoteStore::new();
    let mut graph = LinkGraph::new();
    fetch_from_org_roam_db(&pool, &config, &mut store, &mut graph)?;
    println!(
        "Loaded {} notes, {} graph nodes, {} edges",
        store.len(),
        graph.node_count(),
        graph.edge_count()
    );
    flush_notes(&pool, &mut store)?;
    flush_links(&pool, &mut graph)?;
    Ok(())
}
