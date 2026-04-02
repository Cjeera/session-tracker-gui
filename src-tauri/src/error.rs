use thiserror::Error;
use serde::{Serialize};
use dotenvy;

#[derive(Error, Debug)]
pub enum AppError 
{
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Time parse error: {0}")]
    TimeParse(#[from] chrono::ParseError),
    
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Var error: {0}")]
    Var(#[from] std::env::VarError),

    #[error("Env error: {0}")]
    Env(#[from] dotenvy::Error),

    #[error("Invalid input")]
    Parse,

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("{0}")]
    Message(String),

    #[error("No Game Found!")]
    NotFound(),
}

impl Serialize for AppError
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where 
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}