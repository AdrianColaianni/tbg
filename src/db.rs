use serde::{Deserialize, Serialize};
use chrono::prelude::{DateTime, Local};
use thiserror::Error;
use std::{fs, io};

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskList {
    pub id: usize,
    pub name: String,
    pub tasks: Box<Vec<Task>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub tags: Box<Vec<String>>,
    pub start_date: DateTime<Local>,
    pub due_date: DateTime<Local>,
}

const DB_PATH: &str = "./data/db.json";

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

pub fn read_db() -> Vec<TaskList> {
    if let Ok(db_content) = fs::read_to_string(DB_PATH) {
        if let Ok(parsed) = serde_json::from_str::<Vec<TaskList>>(&db_content) {
            return parsed;
        }
    }
    // Default list
    let default = vec![
        TaskList {
            id: 0,
            name: "Personal".to_string(),
            tasks: Box::new(vec![
                Task {
                    id: 0,
                    name: "Clean up your room".to_string(),
                    tags: Box::new(vec!["JP".to_string()]),
                    due_date: Local::now(),
                    start_date: Local::now(),
                },
                Task {
                    id: 1,
                    name: "Watch ThePrimeagen".to_string(),
                    tags: Box::new(vec!["rust".to_string()]),
                    due_date: Local::now(),
                    start_date: Local::now(),
                },
            ]),
        },
        TaskList {
            id: 1,
            name: "School".to_string(),
            tasks: Box::new(vec![
                Task {
                    id: 0,
                    name: "Math HW".to_string(),
                    tags: Box::new(vec!["MATH".to_string()]),
                    due_date: Local::now(),
                    start_date: Local::now(),
                },
                Task {
                    id: 1,
                    name: "Smart Book".to_string(),
                    tags: Box::new(vec!["2070".to_string()]),
                    due_date: Local::now(),
                    start_date: Local::now(),
                },
            ]),
        },
    ];
    let db_content = serde_json::to_string(&default).unwrap();
    fs::write(DB_PATH, db_content).unwrap();
    default
}
