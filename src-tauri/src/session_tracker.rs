use crate::error::AppError;
use crate::database_operations::{insert_data, SessionRust};
use chrono::{Utc};
use serde::Serialize;
use std::{thread, time};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use tauri::{AppHandle, Emitter};
use libsw::{Sw, StopwatchImpl};
use std::time::Instant;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StopwatchPayload 
{
    elapsed_ms: u128,
}

/// Normalizes process names for comparison
fn normalize(string: &str) -> String 
{
    // Given string is lowercased, making the process search case-insensitive.
    string.to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Finds a running process by a user inputted name.
pub fn find_process_by_name(game_input: &str) -> Result<Pid, AppError> 
{
    let mut system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

    system.refresh_processes(ProcessesToUpdate::All, true);

    let search_norm = normalize(game_input);

    // Searches all processes for the inputted name and returns the process ID if one is found.
    // The inputted name and the process list have been lowercased and had special characters removed, making the search case-insensitive.
    match system.processes().values().find_map(|p| 
    {
        p.name()
            .to_str()
            .map(|name| normalize(name).contains(&search_norm))
            .and_then(|matches| matches.then_some(p.pid()))
    })
    {
        // If the name is found, it's process ID is returned to search_processes, then the frontend. If not, error message is returned.
        Some(pid) =>
        {
            Ok(pid)
        }
        None => return Err(AppError::NotFound())
    }
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
    let _ = stopwatch.stop();

    let payload = StopwatchPayload
    {
        elapsed_ms: stopwatch.elapsed().as_millis(),
    };

    // The payload struct is sent to the frontend.
    app.emit("stopwatch-tick", payload).unwrap();
}

/// Tracks a running application's session time.
pub fn track_session(game_input: &String, pid: u32, app: AppHandle) -> Result<SessionRust, AppError>
{
    // Start timestamp is taken.
    let start = Utc::now();

    // Gets a list of all running processes.
    let mut system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));
    
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

    // The data is gathered inot the session_data struct.
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
    
    // The struct data is inserted into the database.
    insert_data(session_data)?;

    Ok(())
}