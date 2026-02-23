use std::{env, error::Error, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ToricelliConfig {
    pub dir: PathBuf,
    pub db: PathBuf,
    pub org_roam_db: PathBuf,
    /// Properties to flush to org file frontmatter.
    /// If empty, flushes all Note fields (stability, score, mtimes).
    pub flush_properties: Vec<String>,
}

impl Default for ToricelliConfig {
    fn default() -> Self {
        let dir = PathBuf::from(env::var("HOME").unwrap()).join(".toricelli/");
        ToricelliConfig {
            db: dir.clone().join("main.db"),
            dir,
            org_roam_db: PathBuf::from(env::var("HOME").unwrap()).join(".emacs.d/org-roam.db"),
            flush_properties: vec![],
        }
    }
}

impl ToricelliConfig {
    pub fn from_str() -> Result<Self, Box<dyn Error>> {
        let content = std::fs::read_to_string("./config.toml")?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_env() -> Self {
        let dir = env::var("TORICELLI_DIR").map_or(
            PathBuf::from(env::var("HOME").unwrap()).join(".toricelli/"),
            PathBuf::from,
        );
        // Empty vec means flush all Note fields (stability, score, mtimes)
        let flush_properties = env::var("TORICELLI_FLUSH_PROPERTIES")
            .map(|s| s.split(',').map(|p| p.trim().to_uppercase()).collect())
            .unwrap_or_else(|_| vec![]);

        let r = Self {
            org_roam_db: env::var("TORICELLI_ORG_ROAM_DB").map_or(
                PathBuf::from(env::var("HOME").unwrap()).join(".emacs.d/org-roam.db"),
                PathBuf::from,
            ),
            db: env::var("TORICELLI_DB").map_or(dir.clone().join("main.db"), PathBuf::from),
            dir,
            flush_properties,
        };
        println!("{:?} {:?} {:?}", r.dir, r.db, r.org_roam_db);
        r
    }

    #[cfg(test)]
    pub fn for_testing(test_name: &str) -> Self {
        let dir = PathBuf::from(format!("/tmp/test_out/{}/", test_name));
        Self {
            dir: dir.clone(),
            org_roam_db: PathBuf::from("./testdata/org_roam.db".to_string()),
            db: dir.clone().join("main.db"),
            flush_properties: vec![], // Empty means all fields
        }
    }
}
