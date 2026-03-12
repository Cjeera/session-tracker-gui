use crate::error::AppError;
use crate::input::{read_line_safe, read_numeric_input};
use crate::csv_fallback::insert_data_fallback;
use chrono::{DateTime, Utc};
use rusqlite::{Connection};
use serde::{Deserialize, Serialize};
use std::fs;

/// A struct for housing select results from the SQLite database.
#[derive(Debug)]
pub struct Session
{
    id: i32,
    game: String,
    start_ts: DateTime<Utc>,
    end_ts: DateTime<Utc>,
    duration_seconds: i64,
    notes: Option<String>,
}

/// A struct for housing session data to be sent to the SQLite database.
#[derive(Serialize, Deserialize)]
pub struct SessionRust
{
    pub game: String,
    pub start_ts: DateTime<Utc>,
    pub end_ts: DateTime<Utc>,
    pub duration_seconds: i64,
    pub notes: Option<String>,
}

/// A struct for housing session data for CSV fallback.
#[derive(Debug, Deserialize)]
struct SessionRecord 
{
    game: String,
    start_ts: String,
    end_ts: String,
    duration_seconds: i64,
    #[serde(default)] 
    notes: Option<String>,
}

pub fn open_connection() -> Result<Connection, rusqlite::Error>
{
    let conn = Connection::open("sessions.db")?;

    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    Ok(conn)
}

/// Maps database rows into Session struct
fn map_row_to_session(row: &rusqlite::Row<'_>) -> Result<Session, rusqlite::Error> 
{
    Ok(Session 
    {
        id: row.get(0)?,
        game: row.get(1)?,
        start_ts: row.get(2)?,
        end_ts: row.get(3)?,
        duration_seconds: row.get(4)?,
        notes: row.get(5)?,
    })
}

/// Handles database query menu
pub fn database_queries(conn: &Connection) -> Result<(), AppError>
{
    println!("1. Select All\n2. Select Specific Row");
    let query_choice = read_line_safe()?;

    match query_choice.as_str()
    {
        "1" =>
        {
            let mut stmt = conn.prepare(
                "SELECT id, game, start_ts, end_ts, duration_seconds, notes FROM sessions"
            )?;

            let rows = stmt.query_map([], map_row_to_session)?;

            for row in rows
            {
                let session = row?;
                println!(
                    "ID = {}, Game = {}, Start = {}, End = {}, Duration = {}, Notes = {}",
                    session.id,
                    session.game,
                    session.start_ts,
                    session.end_ts,
                    session.duration_seconds,
                    session.notes.as_deref().unwrap_or("")
                );
            }
        }

        "2" =>
        {
            let id: i32 = read_numeric_input("Enter Row ID:")?;

            let mut stmt = conn.prepare("SELECT * FROM sessions WHERE id = ?1")?;

            // The search for in the query (?1) is set to the inputted ID.
            let rows = stmt.query_map([id], map_row_to_session)?;

            for row in rows
            {
                let session = row?;
                println!(
                    "ID = {}, Game = {}, Start = {}, End = {}, Duration = {}, Notes = {}",
                    session.id,
                    session.game,
                    session.start_ts,
                    session.end_ts,
                    session.duration_seconds,
                    session.notes.as_deref().unwrap_or("")
                );
            }
        }
        _ =>
        {
            return Err(AppError::Parse);
        }
    }

    Ok(())
}

pub fn create_tables(conn: &Connection) -> Result<(), AppError>
{
    conn.execute_batch(
        "       
        CREATE TABLE IF NOT EXISTS games (
            game_id INTEGER PRIMARY KEY,
            title TEXT UNIQUE NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
            session_id INTEGER PRIMARY KEY,
            game_id INT NOT NULL,
            start_ts TEXT NOT NULL,
            end_ts TEXT NOT NULL,
            duration_seconds INTEGER NOT NULL,
            notes TEXT,
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );

        CREATE TABLE IF NOT EXISTS game_covers (
            game_id INT PRIMARY KEY,
            path TEXT,
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );"
    )?;

    Ok(())
}

/// Inserts session into database.
pub fn insert_data(session_data: SessionRust) -> Result<(), AppError>
{
    let conn = open_connection()?;
    create_tables(&conn)?;

    let start_str = session_data.start_ts.to_rfc3339();
    let end_str = session_data.end_ts.to_rfc3339();

    let db_result = || -> Result<(), AppError>
    {
        conn.execute(
            "INSERT OR IGNORE INTO games (title)
            VALUES (?1)",
        (
            &session_data.game,
        ))?;

            let game_id: i64 = conn.query_row(
        "SELECT game_id
            FROM games
            WHERE title = ?1;", 
        [&session_data.game], |row| row.get(0))?;

        conn.execute(
        "INSERT INTO sessions (game_id, start_ts, end_ts, duration_seconds, notes)
            VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &game_id,
            &start_str,
            &end_str,
            &session_data.duration_seconds,
            &session_data.notes,
        ))?;

        Ok(())

    }();

    match db_result
    {
        Ok(_) => return Ok(()),
        Err(error) =>
        {
            insert_data_fallback(session_data)?;
            return Err(error)
        }
    }
}

/// Imports CSV sessions into the database
pub fn insert_data_from_csv(conn: &mut Connection) -> Result<(), AppError> 
{
    let file_path = "session.csv";
    
    if !std::path::Path::new(file_path).exists() 
    {
        return Ok(());
    }

    let file = fs::File::open(file_path)?;
    
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true) 
        .from_reader(file);

    let tx = conn.transaction()?;

    for result in reader.deserialize() 
    {
        let record: SessionRecord = match result 
        {
            Ok(rec) => rec,
            Err(_) => 
            {
                continue;
            }
        };

        tx.execute(
            "INSERT INTO sessions (game, start_ts, end_ts, duration_seconds, notes)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                record.game.as_str(),
                record.start_ts.as_str(),
                record.end_ts.as_str(),
                record.duration_seconds,
                record.notes,
            ),
        )?;
    }

    tx.commit()?;
    
    Ok(())
}