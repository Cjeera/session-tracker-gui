use crate::error::AppError;
use crate::{RefreshState, SessionState, TrackerStatus, refresh_tray_menu, trigger_refresh};
use crate::database_operations::{insert_data, open_connection, SessionRust, get_process_names};
use chrono::{Utc};
use serde::Serialize;
use std::{thread, time};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use tauri::{AppHandle, Emitter, Manager};
use libsw::{Sw, StopwatchImpl};
use std::time::Instant;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::ffi::OsStr;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StopwatchPayload 
{
    elapsed_ms: u128,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Process
{
    pid: u32,
    name: String,
}

/// Finds a running process by a user inputted name. Returns a vector with Process structs.
pub fn process_search(game_input: &str) -> Result<Vec<Process>, AppError> 
{
    // Gets a list of all running processes.
    let mut system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()))?;

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
                matches.push((found_score, Process { pid: (process.pid().as_u32()), name: (process.name().to_string_lossy().into_owned()) }));
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
        // Process ID from matches vector and name are pushed into search_results.
        search_results.push(Process { pid: entry.1.pid, name: entry.1.name} );
    }

    Ok(search_results)
}

/// Checks if a specific process is still running.
pub fn process_exists(system: &mut System, pid: Pid) -> bool 
{
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    system.process(pid).is_some()
}

/// Function that sends elapsed time in milliseconds to the frontend.
fn stopwatch_tick(app: &AppHandle, stopwatch: &StopwatchImpl<Instant>)
{
    // Gets the elapsed time in milliseconds from the stopwatch.
    let payload = StopwatchPayload
    {
        elapsed_ms: stopwatch.elapsed().as_millis(),
    };

    // The payload struct with the elapsed time is sent to the frontend.
    if let Err(error) = app.emit("stopwatch-tick", payload) { eprintln!("{error}"); }
}

/// Tracks a running application's session time. Returns a struct containing session data.
pub fn track_session(game_input: String, pid: u32, app: AppHandle) -> Result<(), AppError>
{
    let pid = Pid::from_u32(pid);
    let mut system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()))?;
    let session = app.state::<SessionState>();
    {
        let mut status = session.status.lock().unwrap();
        if !matches!(*status, TrackerStatus::Idle) && *status != TrackerStatus::GameDetected(pid.as_u32()) {
            return Err(AppError::Message("A session is already active or awaiting submission".into()));
        }
        if !process_exists(&mut system, pid) {
            *status = TrackerStatus::Idle;
            return Err(AppError::NotFound());
        }
        *status = TrackerStatus::Running;
    }
    let process_start = system.process(pid).map(|p| p.start_time()).unwrap_or(0);
    let start = Utc::now();
    let worker_app = app.clone();
    thread::spawn(move || {
        let app = worker_app;
        let session = app.state::<SessionState>();
        let mut stopwatch = Sw::new_started();
        loop {
            let status = *session.status.lock().unwrap();
            if status == TrackerStatus::Stopping || !process_exists(&mut system, pid)
                || system.process(pid).map(|p| p.start_time()) != Some(process_start) {
                break;
            }
            if status == TrackerStatus::Paused {
                if !stopwatch.is_stopped() { let _ = stopwatch.stop(); }
            } else if status == TrackerStatus::Running {
                if stopwatch.is_stopped() { let _ = stopwatch.start(); }
            }
            stopwatch_tick(&app, &stopwatch);
            thread::sleep(time::Duration::from_secs(1));
        }
        let end = Utc::now();
        let _ = stopwatch.stop();
        stopwatch_tick(&app, &stopwatch);
        let duration_seconds = (stopwatch.elapsed().as_millis() / 1000) as i64;
        // Do not rediscover the same instance if the user manually stopped it.
        session.dismissed.lock().unwrap().insert((pid.as_u32(), process_start));
        *session.status.lock().unwrap() = TrackerStatus::AwaitingSubmission;
        if let Err(error) = refresh_tray_menu(&app) { eprintln!("{error}"); }
        let session_data = SessionRust {
            game: game_input.clone(),
            start_ts: start,
            end_ts: end,
            duration_seconds,
            notes: None,
            process_name: game_input,
        };
        if let Err(error) = app.emit("session-ended", session_data) { eprintln!("{error}"); }
    });
    Ok(())
}

/// Save the completed session before allowing automatic detection again.
pub fn end_session(app: AppHandle, session_notes: &str, mut session_data: SessionRust) -> Result<(), AppError>
{
    let session = app.state::<SessionState>();
    let mut status = session.status.lock().unwrap();
    if *status != TrackerStatus::AwaitingSubmission {
        return Err(AppError::Message("No completed session is awaiting submission".into()));
    }
    session_data.notes = if session_notes.is_empty() { None } else { Some(session_notes.to_string()) };
    let conn = open_connection()?;
    insert_data(&conn, session_data)?;
    trigger_refresh(&app);
    *status = TrackerStatus::Idle;
    Ok(())
}

/// One persistent worker. The switch enables scanning, not thread creation.
pub fn auto_search(app: AppHandle, refresh: Arc<AtomicBool>) -> Result<(), AppError>
{
    let mut system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()))?;
    thread::spawn(move || {
        let mut process_names = Vec::<String>::new();
        let session = app.state::<SessionState>();
        let settings = app.state::<RefreshState>();
        let mut pending_since: Option<Instant> = None;
        loop {
            thread::sleep(time::Duration::from_secs(1));
            if !settings.enabled.load(Ordering::Relaxed) { continue; }
            if refresh.swap(false, Ordering::Relaxed) {
                match get_process_names(None) {
                    Ok(names) => process_names = names,
                    Err(error) => {
                        eprintln!("Process list refresh failed: {error}");
                        refresh.store(true, Ordering::Relaxed);
                        continue;
                    }
                }
            }
            // Release an unacknowledged detection if the frontend never starts it.
            {
                let mut status = session.status.lock().unwrap();
                if matches!(*status, TrackerStatus::GameDetected(_)) {
                    if pending_since.is_some_and(|t| t.elapsed() >= time::Duration::from_secs(10)) {
                        *status = TrackerStatus::Idle;
                    } else { continue; }
                }
                if *status != TrackerStatus::Idle { continue; }
            }
            system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
            let candidate = {
                let mut dismissed = session.dismissed.lock().unwrap();
                dismissed.retain(|(pid, start)| system.process(Pid::from_u32(*pid)).is_some_and(|p| p.start_time() == *start));
                process_names.iter().filter(|name| !name.trim().is_empty()).find_map(|name| {
                    system.processes_by_exact_name(OsStr::new(name)).find(|p| {
                        p.exists() && !dismissed.contains(&(p.pid().as_u32(), p.start_time()))
                    }).map(|p| Process { pid: p.pid().as_u32(), name: p.name().to_string_lossy().into_owned() })
                })
            };
            if let Some(found) = candidate {
                {
                    let mut status = session.status.lock().unwrap();
                    if *status != TrackerStatus::Idle || !settings.enabled.load(Ordering::Relaxed) { continue; }
                    *status = TrackerStatus::GameDetected(found.pid);
                    pending_since = Some(Instant::now());
                }
                if let Err(error) = app.emit("auto-search-game-found", found.clone()) {
                    eprintln!("{error}");
                    let mut status = session.status.lock().unwrap();
                    if *status == TrackerStatus::GameDetected(found.pid) { *status = TrackerStatus::Idle; }
                }
            }
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    #[ignore]
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