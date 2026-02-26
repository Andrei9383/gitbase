use git2::{build::RepoBuilder, Cred, Error, FetchOptions, PushOptions, Remote, RemoteCallbacks, Repository, Signature};
use std::fs::{self, create_dir, create_dir_all};
use std::io::Error as ioError;
use std::env;
use std::{path::Path, path::PathBuf};
use thiserror::Error;

use uuid::Uuid;

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
    url: Option<String>,
}

impl Database {
    pub fn new(path: &Path, url: Option<String>) -> Result<Self, DatabaseError> {
        let repo = match url.clone() {
            Some(url) => {
                if !path.exists() {
                    create_dir_all(path)?;
                }
                else {
                    debug!("Path already exists, opening existing repository");
                    return Ok(Database {
                        repo: Repository::open(path)?,
                        path: path.to_path_buf(),
                        url: Some(url),                    
                    });
                }
                
                let mut callbacks = RemoteCallbacks::new();
                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    Cred::ssh_key(
                        username_from_url.unwrap(),
                        None,
                        std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
                        None,
                    )
                });

                let mut fetch_options = FetchOptions::new();
                fetch_options.remote_callbacks(callbacks);

                let mut builder = RepoBuilder::new();
                builder.fetch_options(fetch_options);

                builder.clone(&url, path)?
            }
            None => {
                // if !path.exists() {
                //     create_dir_all(path)?;
                // }
                Repository::init(path)?
            }
        };

        Ok(Database {
            repo,
            path: path.to_path_buf(),
            url,
        })
    }

    fn commit(&self, collection: &str, id: &str) -> Result<(), DatabaseError> {
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

    pub fn insert<V: Serialize>(
        &self,
        collection: &str,
        id: Option<&str>,
        data: &V,
    ) -> Result<(), DatabaseError> {
        let dir = self.path.join(collection);

        create_dir_all(&dir)?;

        let random_id = Uuid::new_v4().to_string();
        let id = id.unwrap_or(random_id.as_str());

        let file_path = dir.join(format!("{}.json", id));
        let serialized = serde_json::to_string_pretty(data)?;

        fs::write(&file_path, serialized)?;

        self.commit(collection, id)?;

        if self.url.is_some() {
            let mut remote = self.repo.find_remote("origin")?;

            debug!("remote: {:?}", remote.url().unwrap_or("no url"));

            let mut callbacks = RemoteCallbacks::new();

            callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
                None,
            )
            });

            if let Err(e) = remote.push(&["refs/heads/main"],  Some(PushOptions::new().remote_callbacks(callbacks))) {
                debug!("Failed to push to remote: {}", e);
            } else {
                debug!("Successfully pushed to remote");
            }
        }
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

    fn open_collection(&self, collection: &str) -> Result<PathBuf, DatabaseError> {
        let dir = self.path.join(collection);

        if !dir.exists() {
            return Err(DatabaseError::InvalidCollection(
                "invalid or unknown collection".to_string(),
            ));
        }

        Ok(dir)
    }

    pub fn get_document(&self, collection: &str, id: &str) -> Result<(), DatabaseError> {
        let dir = self.open_collection(collection)?;

        Ok(())
    }
}
