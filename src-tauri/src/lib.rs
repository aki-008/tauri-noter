mod api;
mod db;
mod state;

use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use uuid::Uuid;

use api::{AuthResponse, SyncChange};
use state::UserInfo;
use state::{AppState, BackendProcess};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Db(#[from] rusqlite::Error),
    #[error("{0}")]
    Http(String),
    #[error("{0}")]
    Api(String),
    #[error("Not authenticated")]
    NotAuthenticated,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub sync_status: String,
}

fn note_to_response(n: db::Note) -> NoteResponse {
    NoteResponse {
        id: n.id,
        user_id: n.user_id,
        title: n.title,
        content: n.content,
        created_at: n.created_at,
        updated_at: n.updated_at,
        sync_status: n.sync_status,
    }
}

fn get_user_id(state: &AppState) -> Result<String, AppError> {
    let guard = state.user.lock().unwrap();
    guard
        .clone()
        .map(|u| u.id)
        .ok_or(AppError::NotAuthenticated)
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn try_start_backend(jwt_secret: &str, db_url: &str, app: &tauri::App) -> Option<Child> {
    let cmd = |exe: &str| -> Command {
        let mut c = Command::new(exe);
        c.env("JWT_SECRET", jwt_secret);
        c.env("DATABASE_URL", db_url);
        #[cfg(windows)]
        c.creation_flags(CREATE_NO_WINDOW);
        c
    };

    #[cfg(debug_assertions)]
    {
        let backend_dir = std::env::current_dir()
            .unwrap_or_default()
            .parent()
            .map(|p| p.join("backend"))
            .unwrap_or_else(|| PathBuf::from("../backend"));

        if backend_dir.join("main.py").exists() {
            eprintln!("Starting backend via uvicorn");
            match cmd("uvicorn")
                .args(["main:app", "--host", "127.0.0.1", "--port", "8000"])
                .current_dir(&backend_dir)
                .spawn()
            {
                Ok(c) => return Some(c),
                Err(e) => eprintln!("WARNING: uvicorn failed: {}", e),
            }
        }
    }

    // Try resource dir (production bundle)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let exe_path = resource_dir.join("bin").join("backend.exe");
        if exe_path.exists() {
            eprintln!("Starting bundled backend: {:?}", exe_path);
            match cmd(&exe_path.to_string_lossy()).spawn() {
                Ok(c) => return Some(c),
                Err(e) => eprintln!("WARNING: bundled backend failed: {}", e),
            }
        }
    }

    // Try relative path (dev, if exe exists)
    let local_exe = PathBuf::from("bin/backend.exe");
    if local_exe.exists() {
        eprintln!("Starting backend from: {:?}", local_exe);
        match cmd(&local_exe.to_string_lossy()).spawn() {
            Ok(c) => return Some(c),
            Err(e) => eprintln!("WARNING: local backend.exe failed: {}", e),
        }
    }

    // Final uvicorn fallback (non-debug)
    #[cfg(not(debug_assertions))]
    {
        let backend_dir = std::env::current_dir()
            .unwrap_or_default()
            .parent()
            .map(|p| p.join("backend"))
            .unwrap_or_else(|| PathBuf::from("../backend"));

        if backend_dir.join("main.py").exists() {
            eprintln!("Starting backend via uvicorn (fallback)");
            if let Ok(c) = cmd("uvicorn")
                .args(["main:app", "--host", "127.0.0.1", "--port", "8000"])
                .current_dir(&backend_dir)
                .spawn()
            {
                return Some(c);
            }
        }
    }

    eprintln!("WARNING: No backend could be started.");
    None
}

// ── Auth Commands ──

#[tauri::command]
async fn register(
    state: tauri::State<'_, AppState>,
    username: String,
    password: String,
) -> Result<AuthResponse, AppError> {
    let resp = api::register(&state.backend_url, &username, &password).await?;
    *state.jwt.lock().unwrap() = Some(resp.access_token.clone());
    *state.user.lock().unwrap() = Some(UserInfo {
        id: resp.user.id.clone(),
        username: resp.user.username.clone(),
    });
    Ok(resp)
}

#[tauri::command]
async fn login(
    state: tauri::State<'_, AppState>,
    username: String,
    password: String,
) -> Result<AuthResponse, AppError> {
    let resp = api::login(&state.backend_url, &username, &password).await?;
    *state.jwt.lock().unwrap() = Some(resp.access_token.clone());
    *state.user.lock().unwrap() = Some(UserInfo {
        id: resp.user.id.clone(),
        username: resp.user.username.clone(),
    });
    Ok(resp)
}

#[tauri::command]
fn get_auth_state(state: tauri::State<'_, AppState>) -> Result<Option<UserInfo>, AppError> {
    let guard = state.user.lock().unwrap();
    Ok(guard.clone())
}

#[tauri::command]
fn logout(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    if let Ok(uid) = get_user_id(&state) {
        let _ = db::clear_user_notes(&state.db_path, &uid);
    }
    *state.jwt.lock().unwrap() = None;
    *state.user.lock().unwrap() = None;
    Ok(())
}

// ── Local Note CRUD ──

#[tauri::command]
fn get_notes(state: tauri::State<'_, AppState>) -> Result<Vec<NoteResponse>, AppError> {
    let uid = get_user_id(&state)?;
    let notes = db::get_notes(&state.db_path, &uid)?;
    Ok(notes.into_iter().map(note_to_response).collect())
}

#[tauri::command]
fn create_note(
    state: tauri::State<'_, AppState>,
    title: String,
    content: String,
) -> Result<NoteResponse, AppError> {
    let uid = get_user_id(&state)?;
    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let note = db::create_note(&state.db_path, &uid, &id, &title, &content, &now, &now)?;
    Ok(note_to_response(note))
}

#[tauri::command]
fn update_note(
    state: tauri::State<'_, AppState>,
    id: String,
    title: String,
    content: String,
) -> Result<NoteResponse, AppError> {
    let uid = get_user_id(&state)?;
    let now = now_iso();
    let note = db::update_note(&state.db_path, &uid, &id, &title, &content, &now)?;
    Ok(note_to_response(note))
}

#[tauri::command]
fn delete_note(state: tauri::State<'_, AppState>, id: String) -> Result<(), AppError> {
    let uid = get_user_id(&state)?;
    db::delete_note(&state.db_path, &uid, &id)?;
    Ok(())
}

// ── Sync ──

#[tauri::command]
async fn sync_notes(state: tauri::State<'_, AppState>) -> Result<Vec<NoteResponse>, AppError> {
    let jwt = state.jwt.lock().unwrap().clone();
    let token = jwt.ok_or(AppError::NotAuthenticated)?;
    let uid = get_user_id(&state)?;

    let changes = db::get_pending_changes(&state.db_path, &uid)?;
    let sync_changes: Vec<SyncChange> = changes
        .into_iter()
        .map(|(note, deleted)| SyncChange {
            id: note.id,
            title: note.title,
            content: note.content,
            updated_at: note.updated_at,
            deleted,
        })
        .collect();

    let remote_notes = api::sync(&state.backend_url, &token, sync_changes).await?;

    for rn in &remote_notes {
        let note = db::Note {
            id: rn.id.clone(),
            user_id: uid.clone(),
            title: rn.title.clone(),
            content: rn.content.clone(),
            created_at: rn.created_at.clone(),
            updated_at: rn.updated_at.clone(),
            sync_status: String::new(),
        };
        db::upsert_note(&state.db_path, &note, &uid, "synced")?;
    }

    db::mark_synced(&state.db_path, &uid)?;

    let notes = db::get_notes(&state.db_path, &uid)?;
    Ok(notes.into_iter().map(note_to_response).collect())
}

// ── App Entry Point ──

fn kill_backend(app: &tauri::AppHandle) {
    if let Some(bp) = app.try_state::<BackendProcess>() {
        if let Ok(mut guard) = bp.0.lock() {
            if let Some(ref mut child) = *guard {
                let _ = child.kill();
                let _ = child.wait();
            }
            *guard = None;
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("notes.db");
            let db_path_str = db_path.to_string_lossy().to_string();

            db::init_db(&db_path_str).expect("failed to init local database");

            let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_default();
            let backend_db = format!(
                "sqlite+aiosqlite:///{}/note_taker.db",
                app_dir.to_string_lossy().replace('\\', "/")
            );

            let child = try_start_backend(&jwt_secret, &backend_db, app);

            app.manage(BackendProcess(Mutex::new(child)));
            app.manage(AppState {
                jwt: Mutex::new(None),
                user: Mutex::new(None),
                db_path: db_path_str,
                backend_url: "http://127.0.0.1:8000".to_string(),
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. }
            | tauri::WindowEvent::Destroyed = event
            {
                kill_backend(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            register,
            login,
            get_auth_state,
            logout,
            get_notes,
            create_note,
            update_note,
            delete_note,
            sync_notes,
        ]);

    builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                kill_backend(_app_handle);
            }
        });
}
