use gitbase::Database;
use serde::{Deserialize, Serialize};
use std::fs::{self, read};
use std::{path::Path, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    age: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    name: String,
    completed: bool,
}

fn read_string() -> String {
    use std::io::{self, Read};

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Failed to read from stdin");
    buffer
}

fn read_number() -> u32 {
    use std::io::{self, Read};

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Failed to read from stdin");
    buffer.trim().parse::<u32>().expect("Please enter a valid number")
}

fn print_menu() {
    println!("1. Add Task");
    println!("2. List Tasks");
    println!("3. Exit");
    println!("Enter your choice:");
}


fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("Task Example Tester");

    let test_path = Path::new("tasks");

    let db = Database::new(test_path, Some("git@github.com:Andrei9383/gitbase-polygon.git".to_string())).unwrap();


    loop {
        print_menu();
        let choice = read_number();

        println!("");

        match choice {
            1 => {
                println!("Enter task name:");
                let task_name = read_string().trim().to_string();
                println!("Task '{}' added!", task_name);

                let task = Task {
                    name: task_name,
                    completed: false,
                };

                db.insert("tasks", None, &task).unwrap();

            }
            2 => {
                let tasks = db.get_collection("tasks").unwrap_or_else(|_| Vec::new());
                if tasks.is_empty() {
                    println!("No tasks found.");
                } else {
                    println!("Tasks:");
                    for (index, task) in tasks.iter().enumerate() {
                        println!("{}. {}", index + 1, task);
                    }
                }
            }
            3 => {
                println!("Exiting...");
                break;
            }
            _ => println!("Invalid choice, please try again."),
        }

        println!("");
    }
}
