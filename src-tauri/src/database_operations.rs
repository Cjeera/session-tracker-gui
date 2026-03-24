use crate::error::AppError;
use crate::csv_fallback::insert_data_fallback;
use chrono::{DateTime, Utc};
use rusqlite::{Connection};
use serde::{Deserialize, Serialize};
use std::fs;

/// A struct for housing select results from the SQLite database to be sent to the frontend.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session
{
    session_id: i64,
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
struct SessionCSV 
{
    game: String,
    start_ts: String,
    end_ts: String,
    duration_seconds: i64,
    #[serde(default)] 
    notes: Option<String>,
}

/// A struct for storing game IDs, game titles and cover art.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game
{
    game_id: i64,
    title: String,
    cover_path: Option<String>,
}

/// A struct for play time info on a specific game.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStats
{
    game_id: i64,
    total_playtime: i64,
    total_sessions: i64,
    last_played: Option<String>,
}

/// Opens and returns a sqlite connection.
pub fn open_connection() -> Result<Connection, rusqlite::Error>
{
    let conn = Connection::open("sessions.db")?;

    // Enables foreign keys.
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    Ok(conn)
}

/// Maps database rows into Session struct
fn map_sessions(row: &rusqlite::Row<'_>) -> Result<Session, rusqlite::Error> 
{
    Ok(Session 
    {
        session_id: row.get(0)?,
        start_ts: row.get(1)?,
        end_ts: row.get(2)?,
        duration_seconds: row.get(3)?,
        notes: row.get(4)?,
    })
}

/// Gets all sessions for a specific game via it's ID.
pub fn get_sessions(game_id: i64) -> Result<Vec<Session>, AppError>
{
    let conn = open_connection()?;

    let mut query = conn.prepare(
        "SELECT
            session_id,
            start_ts,
            end_ts,
            duration_seconds,
            notes
        FROM sessions
        WHERE game_id = ?1")?;
    
    // Maps the rows to a session struct, then pushes structs into a vector.
    let sessions = query.query_map([&game_id], map_sessions)?.collect::<Result<_,_>>()?;

    Ok(sessions)
}

/// Maps database rows to game struct.
fn map_games(row: &rusqlite::Row<'_>) -> Result<Game, rusqlite::Error>
{
    Ok(Game 
    {
        game_id: row.get(0)?,
        title: row.get(1)?,
        cover_path: row.get(2)?,
    })
}

/// Gets all games in the database and the path to it's cover art.
pub fn get_games() -> Result<Vec<Game>, AppError>
{
    let conn = open_connection()?;

    let mut query = conn.prepare(
        "SELECT 
            games.game_id, 
            games.title,
            game_covers.path
        FROM games
        LEFT JOIN game_covers ON
        games.game_id = game_covers.game_id;")?;

    // Maps the rows to a Game struct, then pushes structs into a vector.
    let games: Vec<Game> = query.query_map([], map_games)?.collect::<Result<_,_>>()?;

    Ok(games)
}

/// Maps database rows to game stats struct.
fn map_game_stats(row: &rusqlite::Row<'_>) -> Result<GameStats, rusqlite::Error>
{
    Ok(GameStats 
    {
        game_id: row.get(0)?,
        total_playtime: row.get(1)?,
        total_sessions: row.get(2)?,
        last_played: row.get(3)?,
    })
}

/// Gets total playtime, total sessions and the timestamp of the previous session.
pub fn get_stats(game_id: i64) -> Result<GameStats, AppError>
{
    let conn = open_connection()?;

    let mut query = conn.prepare(
        "SELECT
            game_id,
            COALESCE(SUM(duration_seconds), 0),
            COUNT(session_id),
            MAX(start_ts)
        FROM sessions
        WHERE game_id = ?1;")?;
    
    let mut game_stats = query.query_row([&game_id], map_game_stats)?;

    // If last played is empty, meaning the user hasn't played it, then "Not Played" is assinged to last_played.
    if game_stats.last_played.is_none()
    {
        game_stats.last_played = Some("Not Played".to_string());
    }

    Ok(game_stats)
}

pub fn get_game_by_id(game_id: i64) -> Result<Game, AppError>
{
    let conn = open_connection()?;

    let mut query = conn.prepare(
        "SELECT 
            games.game_id, 
            games.title,
            game_covers.path
        FROM games
        LEFT JOIN game_covers ON games.game_id = game_covers.game_id
        WHERE games.game_id = ?1;"
    )?;

    let game = query.query_row([&game_id], map_games)?;

    Ok(game)

}

/// Creates the tables used in the program.
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
            game_id INTEGER NOT NULL,
            start_ts TEXT NOT NULL,
            end_ts TEXT NOT NULL,
            duration_seconds INTEGER NOT NULL,
            notes TEXT,
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );

        CREATE TABLE IF NOT EXISTS game_covers (
            game_id INTEGER PRIMARY KEY,
            path TEXT,
            FOREIGN KEY (game_id) REFERENCES games(game_id) ON DELETE CASCADE
        );"
    )?;

    Ok(())
}

/// Inserts session into database.
pub fn insert_data(conn: &Connection, session_data: SessionRust) -> Result<(), AppError>
{
    if session_data.game.trim().is_empty()
    {
        return Err(AppError::Message("title cannot be empty!".to_string()))
    }

    if session_data.duration_seconds.is_negative()
    {
        return Err(AppError::Message("Duration cannot be negative!".to_string()))
    }

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
        let record: SessionCSV = match result 
        {
            Ok(rec) => rec,
            Err(_) => 
            {
                continue;
            }
        };

        tx.execute(
            "INSERT INTO sessions (game_id, start_ts, end_ts, duration_seconds, notes)
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

#[cfg(test)]
mod tests
{
    use super::*;
    use chrono::TimeZone;
    
    #[test]
    fn test_insert_good_data()
    {
        let conn = Connection::open_in_memory().unwrap();
        // Struct with empty session notes.
        let good_session_full = SessionRust 
        {
            game: String::from("Cyberpunk 2077"),
            start_ts: Utc.with_ymd_and_hms(2023, 10, 1, 14, 0, 0).unwrap(),
            end_ts: Utc.with_ymd_and_hms(2023, 10, 1, 16, 0, 0).unwrap(),
            duration_seconds: 7200,
            notes: Some(String::from("Completed main questline")),
        };

        // Struct with populated session notes.
        let good_session_minimal = SessionRust 
        {
            game: String::from("Stardew Valley"),
            start_ts: Utc.with_ymd_and_hms(2023, 10, 2, 9, 0, 0).unwrap(),
            end_ts: Utc.with_ymd_and_hms(2023, 10, 2, 9, 30, 0).unwrap(),
            duration_seconds: 1800,
            notes: None,
        };
        
        // Both session data structs are inserted to database.
        let result_one = insert_data(&conn, good_session_full);
        let result_two = insert_data(&conn, good_session_minimal);
        
        // Checks that both were successfull
        assert!(result_one.is_ok());
        assert!(result_two.is_ok());
    }

    #[test]
    fn test_insert_empty_title()
    {
        let conn = Connection::open_in_memory().unwrap();

        // Struct with an empty title
        let edge_case_empty_title = SessionRust 
        {
            game: String::from(""), 
            start_ts: Utc::now(),
            end_ts: Utc::now(),
            duration_seconds: 0,
            notes: None,
        };

        // Struct data is inserted into database, error expected.
        let result_one = insert_data(&conn, edge_case_empty_title);

        // Checks if error returned.
        assert!(result_one.is_err());
    }

    #[test]
    fn test_insert_minus_int()
    {
        let conn = Connection::open_in_memory().unwrap();

        let bad_session_negative_time = SessionRust 
        {
            game: String::from("Tenet: The Game"),
            start_ts: Utc.with_ymd_and_hms(2023, 10, 5, 12, 0, 0).unwrap(),
            end_ts: Utc.with_ymd_and_hms(2023, 10, 5, 10, 0, 0).unwrap(), 
            duration_seconds: -7200, 
            notes: Some(String::from("Time is moving backwards")),
        };

        let result_one = insert_data(&conn, bad_session_negative_time);

        assert!(result_one.is_err());
    }

    #[test]
    fn test_duration_zero()
    {
        let conn = Connection::open_in_memory().unwrap();

        let edge_case_zero_duration = SessionRust 
        {
            game: String::from("Accidental Launch"),
            start_ts: Utc.with_ymd_and_hms(2023, 10, 6, 8, 0, 0).unwrap(),
            end_ts: Utc.with_ymd_and_hms(2023, 10, 6, 8, 0, 0).unwrap(),
            duration_seconds: 0,
            notes: None,
        };

        let result_one = insert_data(&conn, edge_case_zero_duration);

        assert!(result_one.is_ok());
    }

    #[test] 
    fn insert_large_string()
    {
        let conn = Connection::open_in_memory().unwrap();
        
        let edge_case_huge_string = SessionRust 
        {
            game: String::from("Skyrim"),
            start_ts: Utc.with_ymd_and_hms(2023, 10, 7, 18, 0, 0).unwrap(),
            end_ts: Utc.with_ymd_and_hms(2023, 10, 7, 20, 0, 0).unwrap(),
            duration_seconds: 7200,
            notes: Some("A".repeat(10_000)),
        };

        let result_one = insert_data(&conn, edge_case_huge_string);

        assert!(result_one.is_ok());
    }
}