use std::{path::Path, path::PathBuf};

use git2::{Error, Repository};

pub mod stores;

use stores::Store;

use crate::stores::default_store::DefaultStore;

pub struct Database<T: Store> {
    repo: Repository,
    path: PathBuf,
    pub store: T,
}

impl<T: Store> Database<T> {
    pub fn new(path: &Path, store: T) -> Result<Self, Error> {
        let repo = Repository::init(path)?;

        let db = Database {
            repo,
            path: path.to_path_buf(),
            store,
        };

        Ok(db)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn database_new() {
        let test_path = Path::new("test_dir");

        if test_path.exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        let store = DefaultStore {
            name: "DefaultStore".to_owned(),
        };
        let db = Database::new(test_path, store).expect("Create database");

        assert!(test_path.exists());
        assert!(test_path.join(".git").exists());
        assert_eq!(db.path, test_path);

        fs::remove_dir_all(test_path).expect("Remove dir");
    }
}
