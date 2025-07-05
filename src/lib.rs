use organic::parser::parse_file;
use organic::types::{Document, Heading, StandardProperties};

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct ID(String);

impl From<&str> for ID {
    fn from(s: &str) -> Self {
        ID(s.to_string())
    }
}

#[derive(Clone, Debug)]
struct Note {
    id: ID,
    mtimes: Vec<DateTime<Utc>>,
    stability: f64,
    links: HashSet<ID>,
    backlinks: HashSet<ID>,
    score: f64,
}

impl Default for Note {
    fn default() -> Self {
        Note {
            id: ID("".to_string()),
            mtimes: vec![],
            stability: 1.0,
            links: HashSet::new(),
            backlinks: HashSet::new(),
            score: 0.0,
        }
    }
}


