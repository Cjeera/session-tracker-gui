use crate::error::AppError;
use crate::database_operations::{insert_data, open_connection, SessionRust};
use chrono::{Utc};
use serde::Serialize;
use std::{thread, time};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use tauri::{AppHandle, Emitter};
use libsw::{Sw, StopwatchImpl};
use std::time::Instant;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Process
{
    pid: u32,
    name: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StopwatchPayload 
{
    elapsed_ms: u128,
}

/// Helper function for removing extension from process names
fn normalize(string: &str) -> String
{
    // Makes a PathBuf from string.
    let string_path = PathBuf::from(string);

    // Non extension portion of string_path is taken, then converted to string and returned.
    string_path.file_stem().unwrap_or_default().to_str().unwrap_or_default().to_string()
}

/// Finds a running process by a user inputted name. Returns a vector with Process structs.
pub fn process_search(game_input: &str) -> Result<Vec<Process>, AppError> 
{
    // Gets a list of all running processes.
    let mut system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

    // Refreshes the process list.
    system.refresh_processes(ProcessesToUpdate::All, true);

    // A fuzzy matcher is created.
    let matcher = SkimMatcherV2::default().smart_case();

    // A vector that contains an integer for storing the score of the found result, and a process struct. 
    let mut matches: Vec<(i64, Process)> = Vec::new();

    // Loops through list of processes
    for process in system.processes().values()
    {
        // Process ID and process name is added to vector if it matches.
        match matcher.fuzzy_match(process.name().to_str().unwrap_or_default(), game_input)
        {
            Some(found_score) =>
            {
                matches.push((found_score, Process { pid: (process.pid().as_u32()), name: (process.name().to_str().unwrap().to_string()) }));
            }       
            None => continue     
        };
    }

    if matches.is_empty()
    {
        return Err(AppError::NotFound())
    }
    
    // The results are sorted into descending order by score, and are then truncated to only keep the top 10 results.
    matches.sort_by(|a, b| b.0.cmp(&a.0));
    matches.truncate(10);

    // A vector contaning process structs is created.
    let mut search_results: Vec<Process> = Vec::new();

    // Loops through matches struct.
    for entry in matches
    {
        // The extension is removed from the process name.
        let name_norm = normalize(&entry.1.name);

        // Process ID from matches vector and normalised name are pushed into search_results.
        search_results.push(Process { pid: entry.1.pid, name: name_norm} );
    }

    Ok(search_results)
}

/// Checks if a specific process is still running.
fn process_exists(system: &mut System, pid: Pid) -> bool 
{
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    system.process(pid).is_some()
}

/// Function that sends elapsed time in milliseconds to the frontend.
fn stopwatch_tick(app: &AppHandle, stopwatch: &StopwatchImpl<Instant>)
{
    let payload = StopwatchPayload
    {
        elapsed_ms: stopwatch.elapsed().as_millis(),
    };

    // The payload struct is sent to the frontend.
    app.emit("stopwatch-tick", payload).unwrap();
}

/// Function that stops the stopwatch.
fn stopwatch_stop(app: &AppHandle, stopwatch: &mut StopwatchImpl<Instant>)
{
    // Stops the stopwatch
    let _ = stopwatch.stop();

    // Gets the elapsed time in milliseconds
    let payload = StopwatchPayload
    {
        elapsed_ms: stopwatch.elapsed().as_millis(),
    };

    // The payload struct is sent to the frontend.
    app.emit("stopwatch-tick", payload).unwrap();
}

/// Tracks a running application's session time. Returns a struct containing session data.
pub fn track_session(game_input: &String, pid: u32, app: AppHandle) -> Result<SessionRust, AppError>
{
    // Gets a list of all running processes.
    let mut system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

    // Returns early if process doesn't exist
    if !process_exists(&mut system, Pid::from_u32(pid))
    {
        return Err(AppError::NotFound())
    }

    // Start timestamp is taken.
    let start = Utc::now();
 
    // Sets the amount of time that the thread will sleep for.
    let sleep_time = time::Duration::from_secs(1);

    // A new thread is created which contains the process exists check. 
    // Thread sleeps for 1 second, then checks if found process is still running. Loop repeats if still running.
    let tracker_thread = thread::spawn(move || 
    {
        // A stopwatch is created that will be sent to the frontend.
        let mut stopwatch = Sw::new_started();

        let pid = Pid::from_u32(pid);
        loop 
        {
            thread::sleep(sleep_time);

            if !process_exists(&mut system, pid) 
            {
                break;
            }

            stopwatch_tick(&app, &stopwatch);        
        }

        // Once the application is exited, the stopwatch is stopped.
        stopwatch_stop(&app, &mut stopwatch);
    });

    // Rust waits for the thread to finish, meaning the game has been exited.
    let _ = tracker_thread.join();

    // End timestamp is taken.
    let end = Utc::now();

    // The duration between the start and end in seconds is calculated.
    let duration_seconds = (end - start).num_seconds().max(0);

    // The data is gathered into the session_data struct.
    let session_data = SessionRust
    {
        game: game_input.to_string(),
        start_ts: start,
        end_ts: end,
        duration_seconds: duration_seconds,
        notes: None,
    };

    Ok(session_data)
}

pub fn end_session(session_notes: &str, mut session_data: SessionRust) -> Result<(), AppError>
{
    // Sets sessions notes to None if empty, assigns Some(session_notes) to the struct field if not. 
    if session_notes.is_empty()
    {
        session_data.notes = None;
    }
    else 
    {
        session_data.notes = Some(session_notes.to_string());   
    }

    let conn = open_connection()?;
    
    // The struct data is inserted into the database.
    insert_data(&conn, session_data)?;

    Ok(())
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_normalise()
    {
        assert_eq!(normalize("Game.exe"), "Game");
        assert_eq!(normalize("GAME.exe"), "GAME");
        assert_eq!(normalize("game.exe"), "game");
        assert_eq!(normalize("gamE.exe"), "gamE");
        assert_eq!(normalize("1029.exe"), "1029");
        assert_eq!(normalize("!£*($*%).exe"), "!£*($*%)");
        assert_eq!(normalize("NoExtensionHere"), "NoExtensionHere");
        assert_eq!(normalize("complex.game.name.exe"), "complex.game.name");
    }


    #[test]
    fn test_process_found()
    {
        let result = process_search("System");
        
        assert!(result.is_ok(), "Expected ok, since System always exists and is running");

        match result
        {
            Ok(found_process) =>
            {
                for entry in found_process
                {
                    println!("Found {} with PID {}", entry.name, entry.pid);
                }
            }
            Err(_) => panic!("Expected success!")
        }
    }

    #[test]
    fn test_process_not_found()
    {
        let result = process_search("THISDOESNTEXIST!)!)!!))!");

        assert!(result.is_err(), "Expected an error because the process doesn't exist");

        match result
        {
            Err(AppError::NotFound()) => {},
            _ => panic!("Expected AppError::NotFound, got something else"),
        }
    }
}