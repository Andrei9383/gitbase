use git2::{
    Cred, Error, FetchOptions, PushOptions, Remote, RemoteCallbacks, Repository, Signature,
    build::RepoBuilder,
};
use log::debug;
use std::{path::Path, path::PathBuf};

use crate::DatabaseError;

pub trait SyncEngine {
    fn handle_insert(&self, repo: &Repository, collection: &str, id: &str);

    fn commit(&self, repo: &Repository, collection: &str, id: &str) -> Result<(), DatabaseError> {
        let mut index = repo.index()?;
        let rel_path = Path::new(collection).join(format!("{}.json", id));

        index.add_path(&rel_path)?;
        index.write()?;

        let wt = index.write_tree()?;
        let tree = repo.find_tree(wt)?;

        // TODO: implement custom commiter / email
        let sig = Signature::now("gitbase", "auto@gitbase.com")?;

        let parent_commit = match repo.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => None,
        };

        let parents = match &parent_commit {
            Some(c) => vec![c],
            None => vec![],
        };

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("Update {} / {}", collection, id),
            &tree,
            &parents,
        )?;

        Ok(())
    }

    fn flush(&self);
}

/////////////////////////////////////////////////////////

pub struct NoSync {}

impl SyncEngine for NoSync {
    fn handle_insert(&self, repo: &Repository, collection: &str, id: &str) {}

    fn flush(&self) {}
}

pub struct DefaultSync {
    pub url: String,
}

impl SyncEngine for DefaultSync {
    fn handle_insert(&self, repo: &Repository, collection: &str, id: &str) {
        self.commit(repo, collection, id);

        // if !self.url.is_empty() {
        //     let mut remote = match repo.find_remote("origin") {
        //         Ok(rem) => rem,
        //         Err(_) => return,
        //     };
        //
        //     debug!("remote: {:?}", remote.url().unwrap_or("no url"));
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
    }

    fn flush(&self) {}
}
