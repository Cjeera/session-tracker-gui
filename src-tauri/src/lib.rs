pub mod error;
pub mod session_tracker;
pub mod database_operations;
pub mod api_requests;

use crate::api_requests::{get_cover_art, GameCover};
use crate::session_tracker::{track_session, end_session, process_search, Process, auto_search};
use crate::database_operations::{get_games, get_stats, get_sessions, get_game_by_id, edit_session_notes, insert_cover_art, update_status, get_game_weekly_playtime, get_game_monthly_playtime, get_game_yearly_playtime, get_global_weekly_playtime, get_global_monthly_playtime, get_global_yearly_playtime, get_process_names, insert_process, Session, SessionRust, Game, GameStats, GameTimeRangeStats, GlobalTimeRangeStats};
use crate::error::AppError;
use tauri::Emitter;
use tauri::{AppHandle, Builder, Manager, tray::{TrayIconBuilder}, menu::{Menu, MenuItem}};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tauri::WindowEvent;
use tauri_plugin_dialog::{
    DialogExt,
    MessageDialogKind,
    MessageDialogButtons
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackerStatus
{
    Idle,
    GameDetected(u32),
    Running,
    Paused,
    Stopping,
    AwaitingSubmission,
}

pub struct SessionState
{
    status: Mutex<TrackerStatus>,
    // Ignore processes from completed sessions until they exit.
    dismissed: Mutex<std::collections::HashSet<(u32, u64)>>,
}

pub struct RefreshState
{
    refresh_auto_search: Arc<AtomicBool>,
    enabled: AtomicBool,
}

pub struct TrayState
{
    pause: MenuItem<tauri::Wry>,
    resume: MenuItem<tauri::Wry>,
    stop: MenuItem<tauri::Wry>,
}

#[tauri::command]
fn get_game_list() -> Result<Vec<Game>, AppError>
{
    match get_games()
    {
        Ok(games) => Ok(games),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn get_game_stats(game_id: i64) -> Result<GameStats, AppError>
{
    match get_stats(game_id)
    {
        Ok(game_stats) => Ok(game_stats),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn get_game_weekly_stats(game_id: i64) -> Result<Vec<GameTimeRangeStats>, AppError>
{
    match get_game_weekly_playtime(game_id)
    {
        Ok(stats) => Ok(stats),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn get_game_monthly_stats(game_id: i64) -> Result<Vec<GameTimeRangeStats>, AppError>
{
    match get_game_monthly_playtime(game_id)
    {
        Ok(stats) => Ok(stats),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn get_game_yearly_stats(game_id: i64) -> Result<Vec<GameTimeRangeStats>, AppError>
{
    match get_game_yearly_playtime(game_id)
    {
        Ok(stats) => Ok(stats),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn get_global_weekly_stats() -> Result<Vec<GlobalTimeRangeStats>, AppError>
{
    match get_global_weekly_playtime()
    {
        Ok(stats) => Ok(stats),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn get_global_monthly_stats() -> Result<Vec<GlobalTimeRangeStats>, AppError>
{
    match get_global_monthly_playtime()
    {
        Ok(stats) => Ok(stats),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn get_global_yearly_stats() -> Result<Vec<GlobalTimeRangeStats>, AppError>
{
    match get_global_yearly_playtime()
    {
        Ok(stats) => Ok(stats),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn get_game_sessions(game_id: i64) -> Result<Vec<Session>, AppError>
{
    match get_sessions(game_id)
    {
        Ok(sessions) => Ok(sessions),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn get_single_game(game_id: i64) -> Result<Game, AppError>
{
    match get_game_by_id(game_id)
    {
        Ok(game) => Ok(game),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn update_game_status(game_id: i64, status: &str) -> Result<(), AppError>
{
    match update_status(game_id, status)
    {
        Ok(()) => {},
        Err(error) => return Err(error),
    }

    Ok(())
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
async fn start_tracker(game_input: String, pid: u32, app: AppHandle) -> Result<(), AppError>
{
    track_session(game_input, pid, app.clone())?;
    if let Err(error) = refresh_tray_menu(&app) {
        eprintln!("Failed to refresh tray: {error}");
    }
    Ok(())
}

#[tauri::command]
fn toggle_pause(app: AppHandle) -> Result<(), AppError>
{
    let session = app.state::<SessionState>();
    {
        let mut status = session.status.lock().unwrap();
        if *status != TrackerStatus::Running { return Ok(()); }
        *status = TrackerStatus::Paused;
    }
    refresh_tray_menu(&app)?;
    app.emit("session-state-change", "paused")?;
    Ok(())
}

#[tauri::command]
fn toggle_resume(app: AppHandle) -> Result<(), AppError>
{
    let session = app.state::<SessionState>();
    {
        let mut status = session.status.lock().unwrap();
        if *status != TrackerStatus::Paused { return Ok(()); }
        *status = TrackerStatus::Running;
    }
    refresh_tray_menu(&app)?;
    app.emit("session-state-change", "resumed")?;
    Ok(())
}

#[tauri::command]
fn toggle_end(app: AppHandle) -> Result<(), AppError>
{
    let session = app.state::<SessionState>();
    {
        let mut status = session.status.lock().unwrap();
        if !matches!(*status, TrackerStatus::Running | TrackerStatus::Paused) {
            return Ok(());
        }
        *status = TrackerStatus::Stopping;
    }
    refresh_tray_menu(&app)?;
    // The tracking thread emits session-ended when it has finalised the data.
    Ok(())
}

#[tauri::command]
fn end_tracker(app: AppHandle, session_notes: &str, session_data: SessionRust) -> Result<(), AppError>
{
    end_session(app, session_notes, session_data)
}

#[tauri::command]
fn edit_notes(session_id: i64, updated_notes: &str) -> Result<(), AppError>
{
    match edit_session_notes(session_id, updated_notes)
    {
        Ok(()) => Ok(()),
        Err(error) => Err(error)
    }
}

#[tauri::command]
async fn fetch_cover_art(name: &str, is_auto_fetch: bool) -> Result<Vec<GameCover>, AppError>
{
    get_cover_art(name, is_auto_fetch).await
}

#[tauri::command]
async fn insert_selected_cover(cover: GameCover, game_id: i64, is_auto_fetch: bool) -> Result<(), AppError>
{
    if let Some(cover_art) = cover.cover
    {
        insert_cover_art(game_id, &cover_art.url, is_auto_fetch)?;      
    }
   
    Ok(())
}

#[tauri::command]
fn insert_process_name(game_id: i64, process_name: &str, app: AppHandle) -> Result<(), AppError>
{
    insert_process(game_id, process_name)?;
    trigger_refresh(&app);

    Ok(())
}

#[tauri::command]
fn get_processes(game_id: Option<i64>) -> Result<Vec<String>, AppError> 
{ 
    Ok(get_process_names(game_id)?)
}

#[tauri::command]
fn set_auto_search(app: AppHandle, enabled: bool)
{
    app.state::<RefreshState>().enabled.store(enabled, Ordering::Relaxed);
    if enabled {
        trigger_refresh(&app);
    } else {
        let session = app.state::<SessionState>();
        let mut status = session.status.lock().unwrap();
        if matches!(*status, TrackerStatus::GameDetected(_)) {
            *status = TrackerStatus::Idle;
        }
    }
}

pub fn trigger_refresh(app: &AppHandle)
{
    app.state::<RefreshState>().refresh_auto_search.store(true, Ordering::Relaxed);
}

pub fn refresh_tray_menu(app: &AppHandle) -> tauri::Result<()>
{
    let session = app.state::<SessionState>();
    let tray = app.state::<TrayState>();
    let status = *session.status.lock().unwrap();
    tray.pause.set_enabled(status == TrackerStatus::Running)?;
    tray.resume.set_enabled(status == TrackerStatus::Paused)?;
    tray.stop.set_enabled(matches!(status, TrackerStatus::Running | TrackerStatus::Paused))?;
    Ok(())
}

fn show_main_window(app: &AppHandle) -> tauri::Result<()>
{
    if let Some(window) = app.get_webview_window("main")
    {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), AppError> {
    let mut builder = Builder::default();

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(
                |app, args, _cwd|
                {
                    if !args.iter().any(|arg| arg == "--background")
                    {
                        let _ = show_main_window(app);
                    }
                }
            ))
            .plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_autostart::Builder::new().args(["--background"]).build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            start_tracker, 
            end_tracker, 
            search_processes, 
            get_game_list, 
            get_game_stats,
            get_game_weekly_stats,
            get_game_monthly_stats,
            get_game_yearly_stats, 
            get_global_weekly_stats,
            get_global_monthly_stats,
            get_global_yearly_stats,
            get_game_sessions, 
            get_single_game,
            toggle_pause,
            toggle_resume,
            toggle_end,
            edit_notes,
            fetch_cover_art,
            insert_selected_cover,
            update_game_status,
            insert_process_name,
            get_processes,
            set_auto_search
        ])
        .setup(|app|
        {
            use tauri_plugin_autostart::ManagerExt;
            app.manage(SessionState {
                status: Mutex::new(TrackerStatus::Idle),
                dismissed: Mutex::new(std::collections::HashSet::new()),
            });
            app.manage(RefreshState {
                refresh_auto_search: Arc::new(AtomicBool::new(true)),
                enabled: AtomicBool::new(false),
            });
            let _ = app.autolaunch().enable();
            let close_i = MenuItem::with_id(app, "close", "Close", true, None::<&str>)?;
            let exit_i = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Pause Tracker", false, None::<&str>)?;
            let resume = MenuItem::with_id(app, "resume", "Resume Tracker", false, None::<&str>)?;
            let stop = MenuItem::with_id(app, "stop", "Stop Tracker", false, None::<&str>)?;
            let menu = Menu::with_items(app, &[&pause, &resume, &stop, &open_i, &close_i, &exit_i])?;
            app.manage(TrayState { pause, resume, stop });

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| {
                    let window = app.get_webview_window("main").unwrap();
                    match event.id.as_ref() {
                        "close" => {
                            let _ = window.close();
                        }
                        "exit" => {
                            let status = *app.state::<SessionState>()
                                .status.lock().unwrap();

                            let warning = match status
                            {
                                TrackerStatus::Running |
                                TrackerStatus::Paused |
                                TrackerStatus::Stopping => {
                                    Some("A session is in progress. Exit without saving?")
                                }

                                TrackerStatus::AwaitingSubmission => {
                                    Some("You have an unsaved session. Exit without saving?")
                                }

                                _ => None
                            };

                            if let Some(warning) = warning
                            {
                                let handle = app.clone();

                                let _ = show_main_window(app);

                                app.dialog()
                                    .message(warning)
                                    .title("Exit Session Tracker")
                                    .kind(MessageDialogKind::Warning)
                                    .buttons(MessageDialogButtons::YesNo)
                                    .show(move |confirmed| {
                                        if confirmed
                                        {
                                            handle.exit(0);
                                        }
                                    });
                            }
                            else
                            {
                                app.exit(0);
                            }
                        }
                        "open" => {
                            let _ = show_main_window(app);
                        }
                        "pause" => { if let Err(e) = toggle_pause(app.clone()) { eprintln!("{e}"); } }
                        "resume" => { if let Err(e) = toggle_resume(app.clone()) { eprintln!("{e}"); } }
                        "stop" => { if let Err(e) = toggle_end(app.clone()) { eprintln!("{e}"); } }
                        _ => {}
                    }
                })
                .build(app)?;

            let background = std::env::args()
                .any(|arg| arg == "--background");

            if !background
            {
                if let Some(window) = app.get_webview_window("main")
                {
                    window.show()?;
                    window.set_focus()?;
                }
            }

            // Exactly one search worker, initially disabled until the UI enables it.
            let refresh = app.state::<RefreshState>().refresh_auto_search.clone();
            auto_search(app.handle().clone(), refresh)?;
            Ok(())
        }).on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event
            {
                if window.label() != "main"
                {
                    return;
                }

                // Prevent the window from being destroyed.
                api.prevent_close();

                let status = *window
                    .state::<SessionState>()
                    .status.lock().unwrap();

                if matches!(
                    status,
                    TrackerStatus::AwaitingSubmission |
                    TrackerStatus::Stopping
                )
                {
                    let window = window.clone();

                    window.app_handle().dialog()
                        .message(
                            "You have an unsaved session. \
                            Hide the application to the system tray?"
                        )
                        .title("Unsaved Session")
                        .kind(MessageDialogKind::Warning)
                        .buttons(MessageDialogButtons::YesNo)
                        .show(move |confirmed| {
                            if confirmed
                            {
                                let _ = window.hide();
                            }
                        });
                }
                else
                {
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}