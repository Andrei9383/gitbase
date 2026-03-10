use git2::{
    Cred, Error, FetchOptions, PushOptions, Remote, RemoteCallbacks, Repository, Signature,
    build::RepoBuilder,
};

pub trait Sync {
    fn on_insert(&self, repo: &Repository);

    fn flush();
}

pub struct NoSync {}

impl Sync for NoSync {
    fn on_insert(&self, repo: &Repository) {}

    fn flush() {}
}

pub struct DefaultSync {
    pub url: String,
}

impl Sync for DefaultSync {
    fn on_insert(&self, repo: &Repository) {
        if self.url.is_some() {
            let mut remote = repo.find_remote("origin")?;

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

            if let Err(e) = remote.push(
                &["refs/heads/main"],
                Some(PushOptions::new().remote_callbacks(callbacks)),
            ) {
                debug!("Failed to push to remote: {}", e);
            } else {
                debug!("Successfully pushed to remote");
            }
        }
    }

    fn flush() {}
}
