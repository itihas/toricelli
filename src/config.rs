use std::{env, path::PathBuf};

#[derive(Clone)]
pub struct ToricelliConfig {
    pub dir: PathBuf,
    pub db: PathBuf,
    pub org_roam_db: PathBuf,
}

impl ToricelliConfig {
    pub fn from_env() -> Self {
	let dir = PathBuf::from(
                env::var("TORICELLI_DIR").unwrap_or_else(|_| "$HOME/.toricelli/".to_string()),
        );
	let r = Self {
            org_roam_db: env::var("TORICELLI_ORG_ROAM_DB")
                .expect("Specify org roam DB from which to import notes")
                .map(PathBuf::from)
                .expect("TORICELLI_ORG_ROAM_DB should be set to a valid path"),
	    db: env::var("TORICELLI_DB").map_or(dir.clone().join("main.db"), PathBuf::from),
            dir,
        };
	println!("{:?} {:?} {:?}", r.dir, r.db, r.org_roam_db);
	r
    }

    #[cfg(test)]
    pub fn for_testing(test_name: &str) -> Self {
        Self {
            dir: PathBuf::from(format!("/tmp/test_out/{}/", test_name)),
            org_roam_db: Some(PathBuf::from("./testdata/org_roam.db".to_string())),
        }
    }
}
