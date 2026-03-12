use crate::{database_operations::SessionRust, error::AppError};
use csv::WriterBuilder;
use std::fs::OpenOptions;

/// Writes session data to CSV file
pub fn insert_data_fallback(session_data: SessionRust) -> Result<(), AppError>
{
    let file_path = "session.csv";
    let path = std::path::Path::new(file_path);

    // Checks if the file contains any data.
    let file_has_data =
        path.exists()
        && std::fs::metadata(path)
            .map(|m| m.len() > 0)
            .unwrap_or(false);

    // Opens the file in append mode, creating it if it doesn't exist.
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_path)?;

    // Creates a CSV writer configured without automatic headers.
    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    // Writes headers to file if it contains no data.
    if !file_has_data
    {
        writer.write_record(&["game", "start_ts", "end_ts", "duration_seconds", "notes"])?;
    }

    let notes_str = session_data.notes.unwrap_or("".to_string());

    let start_str = session_data.start_ts.to_rfc3339();
    let end_str = session_data.end_ts.to_rfc3339();

    // Data is written to the csv file as a new row.
    writer.write_record
    (
        &[
            &session_data.game,
            &start_str,
            &end_str,
            &session_data.duration_seconds.to_string(),
            &notes_str,
        ]
    )?;

    writer.flush()?;
    println!("Session data saved to CSV.");
    Ok(())
}