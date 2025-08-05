use toricelli::config::ToricelliConfig;
use toricelli::*;

fn main() {
    println!("Hello, world!");
    let conf = ToricelliConfig::from_env();
    fetch_notes_from_org_roam_db(conf);
}
