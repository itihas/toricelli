use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlite::Value;
use std::{collections::{HashMap, HashSet}, fmt::Display};

use crate::config::ToricelliConfig;

#[derive(Eq, Hash, PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ID(pub String);

impl From<String> for ID {
    fn from(s: String) -> Self {
        ID(s)
    }
}

impl From<&str> for ID {
    fn from(s: &str) -> Self {
        ID(s.to_string())
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}", self.0.to_string())
    }
}

pub type NoteMap = HashMap<ID, Note>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    #[serde(skip)]
    pub id: ID,
    pub mtimes: Vec<DateTime<Utc>>,
    pub stability: f64,
    pub links: HashSet<ID>,
    pub backlinks: HashSet<ID>,
    pub outlinks: HashSet<String>,
    pub score: f64,
}


impl Note {
    pub fn fetch_notes(_config: ToricelliConfig) -> HashMap<ID, Self> {
	todo!()
    }
}

impl Default for Note {
    fn default() -> Self {
        Note {
            id: ID("".to_string()),
            mtimes: vec![],
            stability: 1.0,
            links: HashSet::new(),
            backlinks: HashSet::new(),
            outlinks: HashSet::new(),
            score: 0.0,
        }
    }
}

impl From<Vec<Value>> for Note {
    fn from(values: Vec<Value>) -> Self {
        // Expects columns: id, data (JSON blob)
        let id = match &values[0] {
            Value::String(s) => ID(s.clone()),
            _ => panic!("Expected string for id"),
        };
        let data = match &values[1] {
            Value::String(s) => s.clone(),
            _ => panic!("Expected string for data"),
        };
        let mut note: Note = serde_json::from_str(&data).expect("Failed to parse JSON data");
        note.id = id;
        note
    }
}
