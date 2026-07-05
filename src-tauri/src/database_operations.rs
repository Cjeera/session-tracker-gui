use crate::error::AppError;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize, Clone)]
pub struct SessionRust
{
    pub game: String,
    pub start_ts: DateTime<Utc>,
    pub end_ts: DateTime<Utc>,
    pub duration_seconds: i64,
    pub notes: Option<String>,
}

/// A struct for storing game IDs, game titles and cover art.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game
{
    game_id: i64,
    title: String,
    cover_path: Option<String>,
    status: String,
}

/// A struct for play time info on a specific game.
#[derive(Serialize)]
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
        status: row.get(3)?,
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
            game_covers.path,
            games.status
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
            game_covers.path,
            games.status
        FROM games
        LEFT JOIN game_covers ON games.game_id = game_covers.game_id
        WHERE games.game_id = ?1;"
    )?;

    let game = query.query_row([&game_id], map_games)?;

    Ok(game)
}

pub fn update_status(game_id: i64, status: &str) -> Result<(), AppError>
{
    let conn = open_connection()?;

    let _ = conn.execute(
        "UPDATE games
            SET status = ?1
        WHERE game_id = ?2;", 
        params![&status, &game_id])?;

    Ok(())
}

/// Creates the tables used in the program.
pub fn create_tables(conn: &Connection) -> Result<(), AppError>
{
    conn.execute_batch(
        "       
        CREATE TABLE IF NOT EXISTS games (
            game_id INTEGER PRIMARY KEY,
            title TEXT UNIQUE NOT NULL,
            status TEXT
        );

        CREATE TABLE IF NOT EXISTS sessions (
            session_id INTEGER PRIMARY KEY,
            game_id INTEGER NOT NULL,
            start_ts TEXT NOT NULL,
            end_ts TEXT NOT NULL,
            duration_seconds INTEGER NOT NULL,
            notes TEXT,
            FOREIGN KEY (game_id) REFERENCES games(game_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS game_covers (
            game_id INTEGER PRIMARY KEY,
            path TEXT UNIQUE,
            FOREIGN KEY (game_id) REFERENCES games(game_id) ON DELETE CASCADE
        );"
    )?;

    Ok(())
}

/// Inserts session into database.
pub fn insert_data(conn: &Connection, session_data: SessionRust) -> Result<(), AppError>
{
    // Returns early if title is empty or duration is negative.
    if session_data.game.trim().is_empty()
    {
        return Err(AppError::Message("title cannot be empty!".to_string()))
    }

    if session_data.duration_seconds.is_negative()
    {
        return Err(AppError::Message("Duration cannot be negative!".to_string()))
    }

    create_tables(conn)?;

    // Converts the timestamps to an RFC3339 formatted string.
    let start_str = session_data.start_ts.to_rfc3339();
    let end_str = session_data.end_ts.to_rfc3339();

    let db_result = || -> Result<(), AppError>
    {
        // Inserts game title into db. Doesn't insert if value already exists.
        conn.execute(
            "INSERT OR IGNORE INTO games (title)
                VALUES (?1)",
        (
            &session_data.game,
        ))?;

        // Gets the id of the game via it's title.
        let game_id: i64 = conn.query_row(
            "SELECT game_id
                FROM games
            WHERE title = ?1;", 
        [&session_data.game], |row| row.get(0))?;

        // Inserts the session data into the db, using the game_id from the previous query,
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
            return Err(error)
        }
    }
}

pub fn edit_session_notes(session_id: i64, updated_notes: &str) -> Result<(), AppError>
{
    let conn = Connection::open("sessions.db")?;
    let mut query = conn.prepare(
        "
        UPDATE sessions
        SET notes = ?1
        WHERE session_id = ?2;
        ")?;

    let _ = query.execute((updated_notes, &session_id));

    Ok(())
}

pub fn insert_cover_art(game_id: i64, cover_url: &str, is_auto_fetch: bool) -> Result<(), AppError>
{
    let conn = Connection::open("sessions.db")?;

    let mut query = if is_auto_fetch
    {
        conn.prepare(
        "
        INSERT INTO game_covers (game_id, path) VALUES (?1, ?2) 
        ON CONFLICT(game_id) DO UPDATE SET path = excluded.path 
        WHERE game_covers.path IS NULL;
        ")? 
    }
    else 
    {
        conn.prepare(
        "
        INSERT INTO game_covers (game_id, path) VALUES (?1, ?2)
        ON CONFLICT(game_id) DO UPDATE SET path = excluded.path;
        ")?
    };

    let _ = query.execute((&game_id, &cover_url))?;
    
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

#[cfg(test)]
mod db_query_tests 
{
    use super::*;

    /// Helper function to create an in-memory DB and create tables.
    fn setup_memory_db() -> Connection 
    {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_tables_execution() 
    {
        let conn = Connection::open_in_memory().unwrap();
        let result = create_tables(&conn);
        assert!(result.is_ok(), "Tables should be created without errors");
    }

    #[test]
    fn test_insert_and_get_stats() 
    {
        let conn = setup_memory_db();
        
        // Inserts a game
        conn.execute("INSERT INTO games (title) VALUES ('Test Game 1')", []).unwrap();
        let game_id = conn.last_insert_rowid();

        // Inserts two sessions for this game
        conn.execute(
            "INSERT INTO sessions (game_id, start_ts, end_ts, duration_seconds, notes) 
             VALUES (?1, '2023-10-01T12:00:00Z', '2023-10-01T13:00:00Z', 3600, 'Beat the first boss')",
             [&game_id]
        ).unwrap();

        conn.execute(
            "INSERT INTO sessions (game_id, start_ts, end_ts, duration_seconds, notes) 
             VALUES (?1, '2023-10-02T12:00:00Z', '2023-10-02T12:30:00Z', 1800, 'Grinding')",
             [&game_id]
        ).unwrap();

        // Replicate the get_stats logic.
        let mut query = conn.prepare(
            "SELECT game_id, COALESCE(SUM(duration_seconds), 0), COUNT(session_id), MAX(start_ts) 
             FROM sessions WHERE game_id = ?1;"
        ).unwrap();
        
        let stats = query.query_row([&game_id], map_game_stats).unwrap();

        // Assert correctness
        assert_eq!(stats.game_id, game_id);
        assert_eq!(stats.total_playtime, 5400); // 3600 + 1800 seconds
        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.last_played.unwrap(), "2023-10-02T12:00:00Z");
    }

    #[test]
    fn test_mapping_games_with_cover() 
    {
        let conn = setup_memory_db();
        
        conn.execute("INSERT INTO games (title) VALUES ('Cover Game')", []).unwrap();
        let game_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO game_covers (game_id, path) VALUES (?1, 'https://example.com/cover.jpg')",
            [&game_id]
        ).unwrap();

        let mut query = conn.prepare(
            "SELECT games.game_id, games.title, game_covers.path 
             FROM games 
             LEFT JOIN game_covers ON games.game_id = game_covers.game_id 
             WHERE games.game_id = ?1;"
        ).unwrap();

        let game = query.query_row([&game_id], map_games).unwrap();
        assert_eq!(game.title, "Cover Game");
        assert_eq!(game.cover_path, Some("https://example.com/cover.jpg".to_string()));
    }

    #[test]
    fn test_get_sessions() 
    {
        let conn = setup_memory_db();
        
        conn.execute("INSERT INTO games (title) VALUES ('Session Test Game')", []).unwrap();
        let game_id = conn.last_insert_rowid();

        // Insert a session
        conn.execute(
            "INSERT INTO sessions (game_id, start_ts, end_ts, duration_seconds, notes) 
             VALUES (?1, '2023-10-01T12:00:00Z', '2023-10-01T13:00:00Z', 3600, 'Original Note')",
             [&game_id]
        ).unwrap();

        // Test fetching the session
        let mut query = conn.prepare("SELECT session_id, start_ts, end_ts, duration_seconds, notes FROM sessions WHERE game_id = ?1").unwrap();
        let sessions: Vec<Session> = query.query_map([&game_id], map_sessions).unwrap().map(Result::unwrap).collect();
        
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].duration_seconds, 3600);
    }

    #[test]
    fn test_edit_session_notes() 
    {
        let conn = setup_memory_db();
        
        conn.execute("INSERT INTO games (title) VALUES ('Notes Game')", []).unwrap();
        let game_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO sessions (game_id, start_ts, end_ts, duration_seconds, notes) 
             VALUES (?1, '2023-10-01T12:00:00Z', '2023-10-01T13:00:00Z', 3600, 'Original Note')",
             [&game_id]
        ).unwrap();
        let session_id = conn.last_insert_rowid();

        // Simulate edit_session_notes
        conn.execute(
            "UPDATE sessions SET notes = ?1 WHERE session_id = ?2",
            ["Updated Note", &session_id.to_string()]
        ).unwrap();

        // Verify the update
        let updated_note: String = conn.query_row(
            "SELECT notes FROM sessions WHERE session_id = ?1",
            [&session_id],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(updated_note, "Updated Note");
    }
}