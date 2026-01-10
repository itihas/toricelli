use std::error::Error;

use toricelli::config::ToricelliConfig;
use toricelli::org_roam_db::fetch_from_org_roam_db;
use toricelli::ConnectionPool;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let config = ToricelliConfig::from_env();
    let pool = ConnectionPool {
        main: sqlite::open(&config.db)?,
	org_roam: sqlite::open(&config.org_roam_db)?
    };
    let (notes, graph) = fetch_from_org_roam_db(&pool, &config)?;
    println!("Loaded {} notes, {} graph nodes, {} edges",
             notes.len(), graph.node_count(), graph.edge_count());

    Ok(())
}
