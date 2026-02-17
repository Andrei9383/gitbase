use gitbase::Database;
use serde::{Deserialize, Serialize};
use std::fs;
use std::{path::Path, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    age: u8,
}

fn main() {
    println!("Hello World from main.rs");

    let test_path = Path::new("test_dir");

    let db = Database::new(test_path).unwrap();

    let user_1 = User {
        name: "User1".to_string(),
        age: 20,
    };

    db.insert("users", "user1", &user_1);
}
