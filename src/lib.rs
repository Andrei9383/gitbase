use git2::{Error, Repository, Signature};
use std::fs::{self, create_dir, create_dir_all};
use std::io::Error as ioError;
use std::{path::Path, path::PathBuf};
use thiserror::Error;

use serde::Serialize;

use log::debug;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git Error: {0}")]
    Git(#[from] git2::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Validation Error: {0}")]
    InvalidCollection(String),
}

pub struct Database {
    repo: Repository,
    path: PathBuf,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, DatabaseError> {
        let repo = Repository::init(path)?;
        Ok(Database {
            repo,
            path: path.to_path_buf(),
        })
    }

    pub fn insert<V: Serialize>(
        &self,
        collection: &str,
        id: &str,
        data: &V,
    ) -> Result<(), DatabaseError> {
        let dir = self.path.join(collection);

        create_dir_all(&dir)?;

        let file_path = dir.join(format!("{}.json", id));
        let serialized = serde_json::to_string_pretty(data)?;

        fs::write(&file_path, serialized)?;

        let mut index = self.repo.index()?;
        let rel_path = Path::new(collection).join(format!("{}.json", id));

        index.add_path(&rel_path)?;
        index.write()?;

        let wt = index.write_tree()?;
        let tree = self.repo.find_tree(wt)?;
        let sig = Signature::now("gitbase", "auto@gitbase.com")?;

        let parent_commit = match self.repo.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => None,
        };

        let parents = match &parent_commit {
            Some(c) => vec![c],
            None => vec![],
        };

        self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("Update {} / {}", collection, id),
            &tree,
            &parents,
        )?;

        Ok(())
    }

    fn remove_suffix(str: &str) -> &str {
        str.trim_end_matches(".json")
    }

    pub fn get_collection(&self, collection: &str) -> Result<Vec<String>, DatabaseError> {
        let dir = self.path.join(collection);

        if !dir.exists() {
            return Err(DatabaseError::InvalidCollection(
                "invalid or unknown collection".to_string(),
            ));
        }

        let mut result: Vec<String> = Vec::new();

        for entry in dir.read_dir().expect("should read from dir") {
            if let Ok(entry) = entry {
                debug!(
                    "{:?}",
                    Database::remove_suffix(entry.file_name().to_str().unwrap())
                );
                result
                    .push(Database::remove_suffix(entry.file_name().to_str().unwrap()).to_string());
            }
        }

        Ok(result)
    }

    pub fn get_document(collection: &str, id: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
}
