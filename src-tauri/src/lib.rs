pub mod error;
pub mod session_tracker;
pub mod database_operations;
pub mod api_requests;

use crate::api_requests::{get_cover_art, GameCover};
use crate::session_tracker::{track_session, end_session, process_search, Process, auto_search};
use crate::database_operations::{get_games, get_stats, get_sessions, get_game_by_id, edit_session_notes, insert_cover_art, update_status, get_game_weekly_playtime, get_game_monthly_playtime, get_game_yearly_playtime, get_global_weekly_playtime, get_global_monthly_playtime, get_global_yearly_playtime, get_process_names, insert_process, Session, SessionRust, Game, GameStats, GameTimeRangeStats, GlobalTimeRangeStats};
use crate::error::AppError;
use tauri::Emitter;
use tauri::{Listener, AppHandle, Builder, Manager, tray::{TrayIconBuilder}, menu::{Menu, MenuItem}};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub struct SessionState 
{
    paused: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
}

pub struct RefreshState
{
    refresh_auto_search: Arc<AtomicBool>,
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
    let pause_state = app.state::<SessionState>().paused.clone();
    let end_state = app.state::<SessionState>().ended.clone();

    end_state.store(false, Ordering::Relaxed);
    pause_state.store(false, Ordering::Relaxed);

    match track_session(
        game_input,
        pid,
        app.clone(),
        pause_state,
        end_state.clone()
    )
    {
        Ok(()) =>
        {
            refresh_tray_menu(&app)?;
            Ok(())
        },
        Err(error) =>
        {
            end_state.store(true, Ordering::Relaxed);
            refresh_tray_menu(&app)?;
            Err(error)
        },
    }
}
#[tauri::command]
fn toggle_pause(app: AppHandle) -> Result<(), AppError>
{
    let pause_state = app.state::<SessionState>().paused.clone();

    pause_state.store(true, Ordering::Relaxed);

    refresh_tray_menu(&app)?;

    app.emit("session-state-changed", "paused")?;

    Ok(())
}

#[tauri::command]
fn toggle_resume(app: AppHandle) -> Result<(), AppError>
{
    let pause_state = app.state::<SessionState>().paused.clone();

    pause_state.store(false, Ordering::Relaxed);

    refresh_tray_menu(&app)?;

    app.emit("session-state-changed", "resumed")?;

    Ok(())
}

#[tauri::command]
fn toggle_end(app: AppHandle) -> Result<(), AppError>
{
    let end_state = app.state::<SessionState>().ended.clone();

    end_state.store(true, Ordering::Relaxed);

    refresh_tray_menu(&app)?;

    Ok(())
}

/// Takes frontend input (session_notes) and the session_data struct and sends it to end_session function.
#[tauri::command]
fn end_tracker(session_notes: &str, session_data: SessionRust) -> Result<(), AppError>
{
    end_session(session_notes, session_data)?;
    Ok(())
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
fn insert_process_name(game_id: i64, process_name: &str) -> Result<(), AppError>
{
    insert_process(game_id, process_name)?;

    Ok(())
}

#[tauri::command]
fn get_processes(game_id: Option<i64>) -> Result<Vec<String>, AppError> 
{ 
    Ok(get_process_names(game_id)?)
}

#[tauri::command]
fn start_auto_search(app: AppHandle) -> Result<(), AppError>
{
    let refresh = app.state::<RefreshState>().refresh_auto_search.clone();
    auto_search(app, refresh)?;
    Ok(())
}

pub fn trigger_refresh(app: &AppHandle)
{
    app.state::<RefreshState>()
        .refresh_auto_search
        .store(true, Ordering::Relaxed);
}

pub fn refresh_tray_menu(app: &AppHandle) -> tauri::Result<()>
{
    let session = app.state::<SessionState>();
    let tray = app.state::<TrayState>();

    let paused = session.paused.load(Ordering::Relaxed);
    let ended = session.ended.load(Ordering::Relaxed);

    tray.pause.set_enabled(!paused && !ended)?;
    tray.resume.set_enabled(paused && !ended)?;
    tray.stop.set_enabled(!ended)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), AppError> {
    let mut builder = Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            start_auto_search,
        ])
        .setup(|app| 
        {
            use tauri_plugin_autostart::ManagerExt;

            app.manage(SessionState 
            {
                paused: Arc::new(AtomicBool::new(false)),
                ended: Arc::new(AtomicBool::new(true)),
            });

            
            app.manage(RefreshState
            {
                refresh_auto_search: Arc::new(AtomicBool::new(false)),
            });

            let autostart_manager = app.autolaunch();
            let _ = autostart_manager.enable();

            let close_i = MenuItem::with_id(app, "close", "Close", true, None::<&str>)?;
            let exit_i = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let pause_tracker_i = MenuItem::with_id(
                app, "pause", "Pause Tracker", false, None::<&str>
            )?;

            let resume_tracker_i = MenuItem::with_id(
                app, "resume", "Resume Tracker", false, None::<&str>
            )?;

            let stop_tracker_i = MenuItem::with_id(
                app, "stop", "Stop Tracker", false, None::<&str>
            )?;
 
            let menu = Menu::with_items(app, &[&pause_tracker_i, &resume_tracker_i, &stop_tracker_i, &open_i, &close_i, &exit_i])?;

            app.manage(TrayState
            {
                pause: pause_tracker_i,
                resume: resume_tracker_i,
                stop: stop_tracker_i,
            });
               
            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| 
                    {
                        let window = app.get_webview_window("main").unwrap();
                        match event.id.as_ref()
                        {
                            "close" => 
                            {
                                if !window.is_minimized().unwrap()
                                {
                                    window.minimize().unwrap();
                                }
                            }

                            "exit" => 
                            {
                                let _ = window.close();
                            }

                            "open" => 
                            {
                                if window.is_minimized().unwrap()
                                {
                                    let _ = window.unminimize();
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }

                            "pause" => 
                            {
                                let _ = toggle_pause(app.app_handle().clone());
                                let _ = app.emit("session-state-change", "paused");
                            }

                            "resume" => 
                            {
                                let _ = toggle_resume(app.app_handle().clone());
                                let _ = app.emit("session-state-change", "resumed");
                            }

                            "stop" => 
                            {
                                let _ = toggle_end(app.app_handle().clone());                   
                                let _ = app.emit("session-state-change", "ended");
                            }

                            _ => {}
                        }
                    })
                .build(app)?;

            let handle = app.handle().clone();

            app.listen("start-auto-search", move |_| {
                start_auto_search(handle.clone()).unwrap();
            });

            start_auto_search(app.handle().clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}