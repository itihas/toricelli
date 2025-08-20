use std::{env, path::PathBuf};

#[derive(Clone)]
pub struct ToricelliConfig {
    pub dir: PathBuf,
    pub db: PathBuf,
    pub org_roam_db: PathBuf,
}

impl ToricelliConfig {
    pub fn from_env() -> Self {
        let dir = env::var("TORICELLI_DIR").map_or(
            { PathBuf::from(env::var("HOME").unwrap()).join(".toricelli/") },
            PathBuf::from,
        );
        let r =
            Self {
                org_roam_db: env::var("TORICELLI_ORG_ROAM_DB").map_or(
                    PathBuf::from(env::var("HOME").unwrap())
                    .join(".emacs.d/org-roam.db"),
                    PathBuf::from,
                ),
                db: env::var("TORICELLI_DB").map_or(dir.clone().join("main.db"), PathBuf::from),
                dir,
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
        }
    }
}
