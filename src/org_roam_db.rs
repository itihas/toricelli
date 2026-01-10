use crate::config::ToricelliConfig;
use crate::types::{LinkGraph, Note, NoteStore, ID};
use crate::ConnectionPool;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};

use lexpr::parse::from_str_elisp;
use lexpr::Value;
use sqlite::State;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub enum PropertyValue {
    Text(String),
    StringList(Vec<String>),
    DateList(Vec<DateTime<Utc>>),
}

fn parse_date(d: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(d, "[%Y-%m-%d %a %H:%M]")
        .map(|ndt| {
            ndt.and_local_timezone(FixedOffset::east_opt(19800).unwrap()) // TODO hardcoded IST. Add this to config instead.
                .unwrap()
                .to_utc()
        })
        .ok()
}
impl From<&Value> for PropertyValue {
    fn from(value: &Value) -> Self {
        let value_string = value.to_string().trim_matches('"').to_string();
        let maybe_list: Option<Box<dyn Iterator<Item = String>>> = {
            if value.is_list() {
                Some(Box::new(value.list_iter().unwrap().filter_map(|v| {
                    v.as_str().map(|s| s.trim_matches('"').to_string())
                })))
            } else if value_string.contains(",") {
                Some(Box::new(
                    value_string
                        .split(",")
                        .map(|s| s.trim_matches('"').to_string()),
                ))
            } else {
                None
            }
        };
        match maybe_list {
            Some(items) => {
                let items_vec: Vec<String> = items.collect();
                match items_vec
                    .clone()
                    .iter()
                    .map(|i| parse_date(i.as_str()))
                    .try_collect::<Vec<DateTime<Utc>>>()
                {
                    Some(res) => PropertyValue::DateList(res),
                    None => PropertyValue::StringList(items_vec),
                }
            }
            None => parse_date(value_string.as_str())
                .map_or(PropertyValue::Text(value_string.clone()), |x| {
                    PropertyValue::DateList(vec![x])
                }),
        }
    }
}

#[derive(Debug)]
pub struct OrgRoamProperties(HashMap<String, PropertyValue>);

impl std::ops::Deref for OrgRoamProperties {
    type Target = HashMap<String, PropertyValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Value> for OrgRoamProperties {
    fn from(v: Value) -> Self {
        Self(
            v.to_vec()
                .unwrap()
                .iter()
                .filter_map(|p| p.as_pair())
                .map(|pair| {
                    (
                        pair.0.to_string().trim_matches('"').to_string(),
                        pair.1.into(),
                    )
                })
                .collect(),
        )
    }
}

pub fn fetch_from_org_roam_db(
    pool: &ConnectionPool,
    _config: &ToricelliConfig,
    store: &mut NoteStore,
    link_graph: &mut LinkGraph,
) -> Result<(), Box<dyn Error>> {
    let connection = &pool.org_roam;
    let nodes_query = "SELECT * FROM \"main\".\"nodes\"";
    let mut nodes_statement = connection.prepare(nodes_query)?;

    while let Ok(State::Row) = nodes_statement.next() {
        let id = nodes_statement.read::<String, _>("id")?;
        let properties = from_str_elisp(
            nodes_statement
                .read::<String, _>("properties")
                .unwrap()
                .as_str(),
        )
        .unwrap();
        let properties_map: OrgRoamProperties = properties.into();

        let mtimes = match properties_map.get("MTIME") {
            Some(k) => match k {
                PropertyValue::DateList(d) => Ok(d.clone()),
                _ => Err("MTIME not a date list!"),
            },
            None => Ok(vec![]),
        }?;

        store.insert(Note {
            id: ID(id),
            mtimes,
            ..Default::default()
        });
    }

    // Fetch internal links and build LinkGraph
    // org-roam stores link types as quoted strings, e.g. '"id"'
    let links_query = "SELECT source, dest FROM \"main\".\"links\" WHERE type = '\"id\"'";
    let links_statement = connection.prepare(links_query)?;

    for row in links_statement.into_iter() {
        let row = row.unwrap();
        let src = ID::from(row.read::<&str, _>("source").trim_matches('"'));
        let dest = ID::from(row.read::<&str, _>("dest").trim_matches('"'));
        link_graph.upsert_link(src, dest, 1.0)
    }

    Ok(())
}
