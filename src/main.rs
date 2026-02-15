use gitbase::Database;
use gitbase::stores::default_store::DefaultStore;
use std::fs;
use std::{path::Path, path::PathBuf};

use gitbase::stores::Store;

fn main() {
    println!("Hello World from main.rs");

    let test_path = Path::new("test_dir");

    let store = DefaultStore {
        name: "DefaultStore".to_owned(),
    };

    let db = Database::new(test_path, store).unwrap();

    db.store.insert();
}
