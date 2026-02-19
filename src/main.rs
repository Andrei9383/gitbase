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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("Hello World from main.rs");

    let test_path = Path::new("test_dir");

    let db = Database::new(test_path).unwrap();

    let user_1 = User {
        name: "User2".to_string(),
        age: 20,
    };

    //db.insert("users", "user2", &user_1);

    match db.get_collection("users") {
        Ok(result) => println!("got: {:#?}", result),
        Err(e) => eprintln!("error: {}", e),
    }
}
