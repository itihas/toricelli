//! Functions for reading and writing org file frontmatter properties using orgize.

use orgize::rowan::ast::AstNode;
use orgize::{Org, SyntaxKind};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Update properties in a file-level property drawer.
/// If no property drawer exists, creates one at the beginning of the file.
/// Returns the updated file content.
pub fn update_properties(
    content: &str,
    updates: &HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
    if updates.is_empty() {
        return Ok(content.to_string());
    }

    let org = Org::parse(content);

    // Find the file-level property drawer (in the zeroth section, before any headline)
    let mut drawer_range = None;

    for node in org.document().syntax().descendants() {
        match node.kind() {
            SyntaxKind::HEADLINE => {
                // Stop at first headline - any property drawer after this is headline-level
                break;
            }
            SyntaxKind::PROPERTY_DRAWER => {
                drawer_range = Some(node.text_range());
                break;
            }
            _ => {}
        }
    }

    match drawer_range {
        Some(range) => {
            // Parse existing properties and merge with updates
            let drawer_text = &content[range.start().into()..range.end().into()];
            let existing = parse_property_drawer(drawer_text);
            let merged = merge_properties(existing, updates.clone());
            let new_drawer = format_property_drawer(&merged);

            let mut result = String::new();
            result.push_str(&content[..range.start().into()]);
            result.push_str(&new_drawer);
            result.push_str(&content[range.end().into()..]);
            Ok(result)
        }
        None => {
            // No existing drawer - insert at beginning of file
            let new_drawer = format_property_drawer(updates);
            Ok(format!("{}\n{}", new_drawer, content))
        }
    }
}

/// Parse a property drawer string into a HashMap.
fn parse_property_drawer(drawer: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();
    for line in drawer.lines() {
        let line = line.trim();
        if line.starts_with(':') && !line.starts_with(":PROPERTIES:") && !line.starts_with(":END:") {
            // Format is :KEY: value
            if let Some(rest) = line.strip_prefix(':') {
                if let Some((key, value)) = rest.split_once(':') {
                    let key = key.trim().to_uppercase();
                    let value = value.trim().to_string();
                    if !key.is_empty() {
                        props.insert(key, value);
                    }
                }
            }
        }
    }
    props
}

/// Merge existing properties with updates, preferring updates.
fn merge_properties(
    existing: HashMap<String, String>,
    updates: HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = existing;
    for (k, v) in updates {
        merged.insert(k, v);
    }
    merged
}

/// Format properties into a property drawer string.
fn format_property_drawer(properties: &HashMap<String, String>) -> String {
    let mut lines = vec![":PROPERTIES:".to_string()];

    // Sort keys for consistent output (ID first, then alphabetical)
    let mut keys: Vec<_> = properties.keys().collect();
    keys.sort_by(|a, b| {
        if *a == "ID" {
            std::cmp::Ordering::Less
        } else if *b == "ID" {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });

    for key in keys {
        if let Some(value) = properties.get(key) {
            lines.push(format!(":{}: {}", key, value));
        }
    }

    lines.push(":END:".to_string());
    lines.join("\n")
}

/// Read a file, update its properties, and write it back.
pub fn update_file_properties(
    path: &Path,
    updates: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let updated = update_properties(&content, updates)?;
    fs::write(path, updated)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_property_drawer() {
        let drawer = ":PROPERTIES:\n:ID: abc-123\n:STABILITY: 1.5\n:END:";
        let props = parse_property_drawer(drawer);
        assert_eq!(props.get("ID"), Some(&"abc-123".to_string()));
        assert_eq!(props.get("STABILITY"), Some(&"1.5".to_string()));
    }

    #[test]
    fn test_format_property_drawer() {
        let mut props = HashMap::new();
        props.insert("SCORE".to_string(), "0.85".to_string());
        props.insert("ID".to_string(), "abc-123".to_string());
        let drawer = format_property_drawer(&props);
        assert!(drawer.starts_with(":PROPERTIES:"));
        assert!(drawer.contains(":ID: abc-123"));
        assert!(drawer.contains(":SCORE: 0.85"));
        assert!(drawer.ends_with(":END:"));
    }

    #[test]
    fn test_update_properties_existing_drawer() {
        let content = ":PROPERTIES:\n:ID: abc-123\n:END:\n\n* Headline";
        let mut updates = HashMap::new();
        updates.insert("SCORE".to_string(), "0.85".to_string());
        let result = update_properties(content, &updates).unwrap();
        assert!(result.contains(":ID: abc-123"));
        assert!(result.contains(":SCORE: 0.85"));
        assert!(result.contains("* Headline"));
    }

    #[test]
    fn test_update_properties_no_drawer() {
        let content = "* Headline\nSome content";
        let mut updates = HashMap::new();
        updates.insert("SCORE".to_string(), "0.85".to_string());
        let result = update_properties(content, &updates).unwrap();
        assert!(result.starts_with(":PROPERTIES:"));
        assert!(result.contains(":SCORE: 0.85"));
        assert!(result.contains("* Headline"));
    }
}
