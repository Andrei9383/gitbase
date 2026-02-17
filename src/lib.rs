use std::fs::{self, create_dir};
use std::{path::Path, path::PathBuf};

use git2::{Error, Repository, Signature};

use serde::Serialize;

pub struct Database {
    repo: Repository,
    path: PathBuf,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let repo = Repository::init(path)?;

        let db = Database {
            repo,
            path: path.to_path_buf(),
        };

        Ok(db)
    }

    pub fn insert<V: Serialize>(
        &self,
        collection: &str,
        id: &str,
        data: &V,
    ) -> Result<(), git2::Error> {
        let dir = Path::join(self.path.as_path(), collection);

        match create_dir(&dir) {
            Ok(_t) => (),
            Err(e) => eprintln!("Error when create_dir: {}", e),
        }

        let file_path = PathBuf::from(dir).join(format!("{}.json", id));

        let serialized = serde_json::to_string_pretty(data).unwrap();

        match fs::write(file_path, serialized) {
            Ok(_t) => println!("Saved record with id: {}", id),
            Err(e) => eprintln!("Error saving record with id: {}, error: {}", id, e),
        }

        let mut index = self.repo.index()?;

        let rel_path = Path::new(collection).join(format!("{}.json", id));

        index.add_path(&rel_path).unwrap();

        index.write().unwrap();

        let wt = index.write_tree().unwrap();

        let tree = self.repo.find_tree(wt).unwrap();

        let parent_commit = self.repo.head().unwrap().peel_to_commit().unwrap();

        let sig = Signature::now("gitbase", "auto@gitbase.com").unwrap();

        self.repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("Update {} / {}", collection, id),
                &tree,
                &[&parent_commit],
            )
            .unwrap();

        println!("Commited succesfully!");

        Ok(())
    }
}
