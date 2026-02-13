use std::{path::Path, path::PathBuf};

use git2::{Error, Repository};

struct Database {
    repo: Repository,
    path: PathBuf,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let repo = Repository::init(path)?;

        let db = Database {
            repo: repo,
            path: path.to_path_buf(),
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

        let db = Database::new(test_path).expect("Create database");

        assert!(test_path.exists());
        assert!(test_path.join(".git").exists());
        assert_eq!(db.path, test_path);

        fs::remove_dir_all(test_path).expect("Remove dir");
    }
}

/*
    #[test]
    fn test_database_new() {
        let test_path = Path::new("test_repo_dir");

        // Ensure clean state
        if test_path.exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        let db = Database::new(test_path).expect("Should successfully create database");

        // Verify directory and .git exist
        assert!(test_path.exists());
        assert!(test_path.join(".git").exists());
        assert_eq!(db.path, test_path);

        // Clean up
        fs::remove_dir_all(test_path).expect("Should be able to remove test directory");
    }
*/
