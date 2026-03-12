use thiserror::Error;
use serde::{Serialize};

#[derive(Error, Debug)]
pub enum AppError 
{
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Time parse error: {0}")]
    TimeParse(#[from] chrono::ParseError),

    #[error("Invalid input")]
    Parse,

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("{0}")]
    Message(String),

    #[error("Game not found!")]
    NotFound()
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