use std::error::Error;

use toricelli::config::ToricelliConfig;
use toricelli::org_roam_db::fetch_from_org_roam_db;
use toricelli::{ConnectionPool, create_resources, flush_links, flush_notes, flush_properties_to_files};
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

    // Flush properties to org file frontmatter
    let flush_result = flush_properties_to_files(&store, &config);
    println!(
        "Flushed properties to {} files ({} skipped, {} errors)",
        flush_result.updated,
        flush_result.skipped,
        flush_result.errors.len()
    );
    for (id, err) in &flush_result.errors {
        eprintln!("  Error updating {}: {}", id, err);
    }

    Ok(())
}
