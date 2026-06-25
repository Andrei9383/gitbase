use git2::{
    Cred, Error, FetchOptions, PushOptions, Remote, RemoteCallbacks, Repository, Signature,
    build::RepoBuilder,
};
use serde::de::DeserializeOwned;
use std::env;
use std::fs::{self, create_dir, create_dir_all};
use std::io::Error as ioError;
use std::{path::Path, path::PathBuf};
use thiserror::Error;

use uuid::Uuid;

use serde::{Deserialize, Serialize};

use log::debug;

use crate::sync::SyncEngine;

mod sync;

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

    #[error("General error: {0}")]
    General(String),
}

pub struct Database {
    path: PathBuf,
    repo: Repository,
    url: Option<String>,
    sync_engine: Box<dyn SyncEngine>,
}

pub enum DestType {
    Local,
    Remote,
}

pub struct DatabaseBuilder {
    path: PathBuf,
    url: String,
    sync_type: Box<dyn SyncEngine>,
    credentials_path: PathBuf,
}

impl Default for DatabaseBuilder {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            url: String::new(),
            sync_type: Box::new(sync::NoSync {}),
            credentials_path: PathBuf::new(),
        }
    }
}

impl DatabaseBuilder {
    pub fn local(mut self, path: PathBuf) -> DatabaseBuilder {
        self.path = path;
        self
    }

    pub fn remote(mut self, url: String) -> DatabaseBuilder {
        self.url = url;
        self
    }

    pub fn credentials(mut self, path: PathBuf) -> DatabaseBuilder {
        self.credentials_path = path;
        self
    }

    pub fn sync_type(mut self, sync_type: Box<dyn SyncEngine>) -> DatabaseBuilder {
        self.sync_type = sync_type;
        self
    }

    pub fn build(self) -> Result<Database, DatabaseError> {
        Database::create(self)
    }
}

impl Database {
    pub fn builder() -> DatabaseBuilder {
        DatabaseBuilder::default()
    }

    fn create(builder: DatabaseBuilder) -> Result<Self, DatabaseError> {
        create_dir_all(builder.path.clone())
            .map_err(|err| DatabaseError::General(format!("failed to create path: {}", err)))?;

        let repo = Repository::init(builder.path.clone()).map_err(|err| {
            DatabaseError::General(format!("failed to create repository: {}", err))
        })?;

        // TODO : handle remotes with credntials

        Ok(Database {
            path: builder.path.to_path_buf(),
            repo,
            url: None,
            sync_engine: builder.sync_type,
        })
    }

    // pub fn new(name: String, path: &Path) -> Result<Self, DatabaseError> { create_dir_all(path)
    //         .map_err(|err| DatabaseError::General(format!("failed to create path: {}", err)))?;
    //
    //     let repo = Repository::init(path).map_err(|err| {
    //         DatabaseError::General(format!("failed to create repository: {}", err))
    //     })?;
    //
    //     Ok(Database {
    //         name,
    //         path: path.to_path_buf(),
    //         repo,
    //         url: None,
    //     })
    // }

    // pub fn new(path: &Path, url: Option<String>) -> Result<Self, DatabaseError> {
    //     let repo = match url.clone() {
    //         Some(url) => {
    //             if !path.exists() {
    //                 create_dir_all(path)?;
    //             } else {
    //                 debug!("Path already exists, opening existing repository");
    //                 return Ok(Database {
    //                     repo: Repository::open(path)?,
    //                     path: path.to_path_buf(),
    //                     url: Some(url),
    //                 });
    //             }
    //
    //             let mut callbacks = RemoteCallbacks::new();
    //             callbacks.credentials(|_url, username_from_url, _allowed_types| {
    //                 Cred::ssh_key(
    //                     username_from_url.unwrap(),
    //                     None,
    //                     std::path::Path::new(&format!(
    //                         "{}/.ssh/id_ed25519",
    //                         env::var("HOME").unwrap()
    //                     )),
    //                     None,
    //                 )
    //             });
    //
    //             let mut fetch_options = FetchOptions::new();
    //             fetch_options.remote_callbacks(callbacks);
    //
    //             let mut builder = RepoBuilder::new();
    //             builder.fetch_options(fetch_options);
    //
    //             builder.clone(&url, path)?
    //         }
    //         None => {
    //             // if !path.exists() {
    //             //     create_dir_all(path)?;
    //             // }
    //             Repository::init(path)?
    //         }
    //     };
    //
    //     Ok(Database {
    //         repo,
    //         path: path.to_path_buf(),
    //         url,
    //     })
    // }

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

    fn deserialize<'a, T>(&self, data: &'a [u8]) -> T
    where
        T: Deserialize<'a>,
    {
        let msg = str::from_utf8(data).unwrap();
        serde_json::from_str::<T>(msg).unwrap()
    }

    pub fn get_document<T>(&self, collection: &str, id: &str) -> Result<T, DatabaseError>
    where
        T: DeserializeOwned,
    {
        let dir = self.open_collection(collection)?;

        let document = fs::read(dir.join(format!("{}.json", id)))?;

        let r = document.as_slice();

        let t = self.deserialize::<T>(r);

        Ok(t)
    }
}

// ORM
impl Database {
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

        self.sync_engine.handle_insert(&self.repo, collection, id);

        // self.sync_engine.commit(&self.repo, collection, id)?;

        // if self.url.is_some() {
        //     let mut remote = self.repo.find_remote("origin")?;
        //
        //     debug!("remote: {:?}", remote.url().unwrap_or("no url"));
        //     // TODO: assertions
        //
        //     let mut callbacks = RemoteCallbacks::new();
        //
        //     callbacks.credentials(|_url, username_from_url, _allowed_types| {
        //         Cred::ssh_key(
        //             username_from_url.unwrap(),
        //             None,
        //             std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
        //             None,
        //         )
        //     });
        //
        //     if let Err(e) = remote.push(
        //         &["refs/heads/main"],
        //         Some(PushOptions::new().remote_callbacks(callbacks)),
        //     ) {
        //         debug!("Failed to push to remote: {}", e);
        //     } else {
        //         debug!("Successfully pushed to remote");
        //     }
        // }
        Ok(())
    }
}
