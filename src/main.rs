use std::error::Error;

use clap::{Parser, Subcommand};
use toricelli::config::ToricelliConfig;
use toricelli::org_roam_db::fetch_from_org_roam_db;
use toricelli::types::{Feed, FeedRequest, LinkGraph, NoteStore, SortOrder, ID};
use toricelli::{
    create_resources, fetch_link_graph, fetch_notes, flush_links, flush_notes,
    flush_properties_to_files, review, ConnectionPool,
};

#[derive(Parser)]
#[command(name = "toricelli", about = "Programmable attention for your notes")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Display the review feed, sorted by score (default)
    Current {
        #[command(flatten)]
        req: FeedRequest,
    },
    /// Display recently edited notes
    Recent {
        #[command(flatten)]
        req: FeedRequest,
    },
    /// Record a review of a note
    Review {
        /// Note ID to mark as reviewed
        id: String,
    },
    /// Visit a note in its source application
    Visit {
        /// Note ID to visit
        id: String,
    },
    /// Import notes from configured data sources
    Import,
    /// Flush property changes back to source files
    Flush,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = ToricelliConfig::from_str()?;
    let pool = ConnectionPool {
        main: sqlite::open(&config.db)?,
        org_roam: sqlite::open(&config.org_roam_db)?,
    };

    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Current {
        req: FeedRequest::default(),
    }) {
        Command::Current { mut req } => {
            // Default to ByScore unless user explicitly passed --sort
            if matches!(req.sort, SortOrder::ByScore) {
                req.sort = SortOrder::ByScore;
            }
            cmd_feed(&req, &pool)
        }
        Command::Recent { mut req } => {
            // Default to ByRecent, but allow --sort override
            if matches!(req.sort, SortOrder::ByScore) {
                req.sort = SortOrder::ByRecent;
            }
            cmd_feed(&req, &pool)
        }
        Command::Review { id } => cmd_review(&id, &pool),
        Command::Visit { id } => cmd_visit(&id, &pool),
        Command::Import => cmd_import(&pool, &config),
        Command::Flush => cmd_flush(&pool, &config),
    }
}

fn cmd_feed(req: &FeedRequest, pool: &ConnectionPool) -> Result<(), Box<dyn Error>> {
    let mut store = NoteStore::new();
    fetch_notes(pool, &mut store)?;
    let feed = Feed::build(&store, &req.sort);

    // TODO: build FeedResponse and serialize as JSON
    for id in feed.page(req.offset, req.count) {
        if let Some(note) = store.get(id) {
            println!("{}\t{:.4}", id, note.score);
        }
    }
    Ok(())
}

fn cmd_review(id: &str, pool: &ConnectionPool) -> Result<(), Box<dyn Error>> {
    let mut store = NoteStore::new();
    fetch_notes(pool, &mut store)?;
    let graph = fetch_link_graph(pool)?;

    let id = ID::from(id);
    review(&id, &mut store, &graph)?;
    flush_notes(pool, &mut store)?;

    println!("Reviewed {}", id);
    Ok(())
}

fn cmd_visit(id: &str, _pool: &ConnectionPool) -> Result<(), Box<dyn Error>> {
    let _id = ID::from(id);
    eprintln!("TODO: visit note {}", id);
    Ok(())
}

fn cmd_import(pool: &ConnectionPool, config: &ToricelliConfig) -> Result<(), Box<dyn Error>> {
    create_resources(pool, config)?;

    let mut store = NoteStore::new();
    let mut graph = LinkGraph::new();
    fetch_from_org_roam_db(pool, config, &mut store, &mut graph)?;
    println!(
        "Loaded {} notes, {} graph nodes, {} edges",
        store.len(),
        graph.node_count(),
        graph.edge_count()
    );

    flush_notes(pool, &mut store)?;
    flush_links(pool, &mut graph)?;
    Ok(())
}

fn cmd_flush(pool: &ConnectionPool, config: &ToricelliConfig) -> Result<(), Box<dyn Error>> {
    let mut store = NoteStore::new();
    fetch_notes(pool, &mut store)?;

    let result = flush_properties_to_files(&store, config);
    println!(
        "Flushed properties to {} files ({} skipped, {} errors)",
        result.updated,
        result.skipped,
        result.errors.len()
    );
    for (id, err) in &result.errors {
        eprintln!("  Error updating {}: {}", id, err);
    }
    Ok(())
}
