pub mod error;
pub mod session_tracker;
pub mod database_operations;
pub mod csv_fallback;

use crate::session_tracker::{track_session, end_session, process_search, Process};
use crate::database_operations::{get_games, get_stats, get_sessions, get_game_by_id, Session, SessionRust, Game, GameStats};
use crate::error::AppError;
use tauri::{AppHandle, Builder, Manager};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub struct PauseState {
  paused: Arc<AtomicBool>,
}

#[tauri::command]
async fn get_game_list() -> Result<Vec<Game>, AppError>
{
    match get_games()
    {
        Ok(games) => Ok(games),
        Err(error) => Err(error),
    }
}

#[tauri::command]
async fn get_game_stats(game_id: i64) -> Result<GameStats, AppError>
{
    match get_stats(game_id)
    {
        Ok(game_stats) => Ok(game_stats),
        Err(error) => Err(error),
    }
}

#[tauri::command]
async fn get_game_sessions(game_id: i64) -> Result<Vec<Session>, AppError>
{
    match get_sessions(game_id)
    {
        Ok(sessions) => Ok(sessions),
        Err(error) => Err(error),
    }
}

#[tauri::command]
async fn get_single_game(game_id: i64) -> Result<Game, AppError>
{
    match get_game_by_id(game_id)
    {
        Ok(game) => Ok(game),
        Err(error) => Err(error),
    }
}

/// Takes frontend input (game_input) and sends it to find_process_by_name function. Returns the process ID as an unsigned integer to the frontend.
#[tauri::command]
async fn search_processes(game_input: String) -> Result<Vec<Process>, AppError>
{
    match process_search(&game_input)
    {
        Ok(search_results) => Ok(search_results),
        Err(error) => Err(error)
    }
}

/// Takes frontend input (game_input), pid and app, and sends it to track_session function. Returns a struct containing session data to the frontend.
#[tauri::command]
async fn start_tracker(game_input: String, pid: u32, app: AppHandle) -> Result<SessionRust, AppError>
{
    let pause_state = app.state::<PauseState>().paused.clone();
    pause_state.store(false, Ordering::Relaxed);
    match track_session(&game_input, pid, app.clone(), pause_state)
    {
        Ok(session_data) => Ok(session_data),
        Err(error) => Err(error),
    }
}

#[tauri::command]
async fn toggle_pause(app: AppHandle) -> Result<(), AppError>
{
    let pause_state = app.state::<PauseState>().paused.clone();
    pause_state.store(true, Ordering::Relaxed);

    Ok(())
}

#[tauri::command]
async fn toggle_resume(app: AppHandle) -> Result<(), AppError>
{
    let pause_state = app.state::<PauseState>().paused.clone();
    pause_state.store(false, Ordering::Relaxed);

    Ok(())
}

/// Takes frontend input (session_notes) and the session_data struct and sends it to end_session function.
#[tauri::command]
async fn end_tracker(session_notes: &str, session_data: SessionRust) -> Result<(), AppError>
{
    end_session(session_notes, session_data)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            start_tracker, 
            end_tracker, 
            search_processes, 
            get_game_list, 
            get_game_stats, 
            get_game_sessions, 
            get_single_game,
            toggle_pause,
            toggle_resume,
        ])
        .setup(|app| 
        {
            app.manage(PauseState 
            {
                paused: Arc::new(AtomicBool::new(false)),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}