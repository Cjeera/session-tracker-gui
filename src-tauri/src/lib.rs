pub mod error;
pub mod session_tracker;
pub mod database_operations;
pub mod input;
pub mod csv_fallback;

use crate::session_tracker::{track_session, end_session, find_process_by_name};
use crate::database_operations::{SessionRust};
use crate::error::AppError;


#[tauri::command]
async fn search_processes(game_input: String) -> Result<u32, AppError>
{
    match find_process_by_name(&game_input)
    {
        Ok(pid) => Ok(pid.as_u32()),
        Err(error) => Err(error)
    }
}

#[tauri::command]
async fn start_tracker(game_input: String, pid: u32) -> Result<SessionRust, AppError>
{
    match track_session(&game_input, pid)
    {
        Ok(session_data) => Ok(session_data),
        Err(error) => Err(error),
    }
}

#[tauri::command]
async fn end_tracker(session_notes: &str, session_data: SessionRust) -> Result<(), AppError>
{
    end_session(session_notes, session_data)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_tracker, end_tracker, search_processes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
