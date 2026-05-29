# Note Taker — Architecture Guide

> **For beginners.** Explains every file, what it does, and how frontend talks to Rust talks to backend.

---

## Table of Contents

1. [The Big Picture](#1-the-big-picture)
2. [File-by-File Breakdown](#2-file-by-file-breakdown)
3. [The Mapping Table](#3-the-mapping-table)
4. [Flow Diagrams](#4-flow-diagrams)
5. [Rust Syntax for Beginners](#5-rust-syntax-for-beginners)
6. [What Lives Where](#6-what-lives-where)
7. [Startup Sequence](#7-startup-sequence)

---

## 1. The Big Picture

The app has **3 layers**. Everything flows through the middle one (Rust).

```
┌────────────────────────────────────────────────────────────┐
│                      FRONTEND (SvelteKit)                   │
│  src/routes/notes/+page.svelte                              │
│  src/routes/login/+page.svelte                              │
│  src/routes/register/+page.svelte                           │
│                                                              │
│  User clicks a button → calls invoke('command_name', args)   │
│                              │                               │
│                              ▼ Tauri IPC (JSON over pipe)     │
└─────────────────────────────┬────────────────────────────────┘
                              │
┌─────────────────────────────▼────────────────────────────────┐
│                      RUST CORE (Tauri)                       │
│                                                              │
│  src-tauri/src/lib.rs   ←── tauri::generate_handler![...]    │
│       │                        registers all commands        │
│       ├──→ calls api.rs  ──→ HTTP to backend (register,      │
│       │                       login, sync)                   │
│       ├──→ calls db.rs   ──→ local SQLite (CRUD)            │
│       └──→ manages state  ──→ AppState (JWT, user, path)     │
│                                                              │
└─────────────────────────────┬────────────────────────────────┘
                              │
         ┌────────────────────┤
         ▼                    ▼
┌──────────────────┐  ┌──────────────────────────────┐
│  LOCAL SQLite    │  │   BACKEND (Python — FastAPI)  │
│  notes.db        │  │                                │
│  (rusqlite)      │  │  api/auth/register        ──┐  │
│  Per-user        │  │  api/auth/login           ──┤  │
│  path: AppData   │  │  api/notes/sync           ──┘  │
│                  │  │                                │
└──────────────────┘  │  ┌──────────┐                  │
                      │  │ note_taker.db (aiosqlite)   │
                      │  │ Per-user path: AppData      │
                      └──┴─────────────────────────────┘
```

### Simple Analogy

| Layer | Analogy | Job |
|-------|---------|-----|
| **Frontend** | The waiter | Takes your order, shows you the food |
| **Rust** | The chef | Actually makes the food, keeps the kitchen clean |
| **Python Backend** | The cloud kitchen | Makes food when the chef needs help (sync) |
| **Local SQLite** | The chef's notebook | Keeps track of orders locally |
| **Backend DB** | The cloud database | Holds a copy in case you use another device |

---

## 2. File-by-File Breakdown

### 2A. Frontend Files

#### `src/routes/+layout.ts`

```typescript
export const prerender = true;   // Build all pages at compile time
export const ssr = false;         // Don't run on a server (Tauri has none)
```

**What it does:** Configures SvelteKit for Tauri. Since Tauri doesn't have a Node.js server, every page must be pre-built (static) and run 100% in the user's browser.

---

#### `src/routes/login/+page.svelte`

```svelte
<script lang="ts">
  // This import is the KEY to talking to Rust:
  import { invoke } from '@tauri-apps/api/core';

  async function handleLogin(e: Event) {
    const resp: any = await invoke('login', { username, password });
    //            calls Rust ────┘              └─── arguments sent as JSON
    auth.login(resp.user, resp.access_token);  // save token in store
    goto('/notes');                            // navigate to notes page
  }
</script>
```

**What it does:**
- Shows a login form (username + password fields)
- User clicks Login → calls `invoke('login', ...)`
- `invoke()` sends the data to Rust via Tauri IPC (a fancy JSON pipe)
- Rust talks to the Python backend, gets back a JWT token
- Token + user info gets saved in the auth store
- Navigates to the notes page

**Key idea:** `invoke('command_name', {arg1: val1, arg2: val2})` is how the frontend talks to Rust. The `command_name` must match a `#[tauri::command]` function in `lib.rs`.

---

#### `src/routes/register/+page.svelte`

Same pattern as login. Calls `invoke('register', { username, password })`. Returns JWT token.

---

#### `src/routes/notes/+page.svelte`

This is the main app. It calls **5 different invoke commands**:

| Line | Code | What it does |
|------|------|-------------|
| 25 | `invoke('get_notes')` | Load notes from local DB |
| 50 | `invoke('update_note', { id, title, content })` | Save edited note |
| 58 | `invoke('create_note', { title, content })` | Create new note |
| 72 | `invoke('delete_note', { id })` | Delete a note |
| 87 | `invoke('sync_notes')` | Sync with backend |

**What it does:**
- Left sidebar: list of notes (click to select)
- Right panel: editor (title + content inputs)
- Auto-saves on input change (`oninput={saveNote}`)
- Sync button (refresh icon) sends pending changes to the backend
- Logout button clears everything

---

#### `src/lib/stores/auth.ts`

```typescript
import { writable, derived } from 'svelte/store';

const { subscribe, set } = writable<AuthState>(initial);

export const auth = {
  login(user, token) { set({ user, token }); localStorage.setItem('auth', ...); },
  logout()           { set({ user: null, token: null }); localStorage.removeItem('auth'); },
};

export const isLoggedIn = derived(auth, ($a) => $a.user !== null);
```

**What it does:**
- Holds the current user's info and JWT token
- Saves to `localStorage` so login survives app restarts
- `isLoggedIn` derived store — reactively true/false based on auth state

**Key idea for Rust beginners:** Svelte stores are like reactive variables. When `auth.login()` is called, every component watching `isLoggedIn` re-renders automatically.

---

#### `src/lib/stores/notes.ts`

```typescript
function createNotes() {
  const { subscribe, set, update } = writable<Note[]>([]);
  return {
    subscribe, set, update,
    updateNote(updated) { update(ns => ns.map(n => n.id === updated.id ? updated : n)); },
    removeNote(id)      { update(ns => ns.filter(n => n.id !== id)); },
    reset()             { set([]); },
  };
}
export const notes = createNotes();
```

**What it does:**
- Holds the note list in memory
- `updateNote(note)` — replace a note by ID
- `removeNote(id)` — remove a note by ID
- `reset()` — clear all notes (on logout)

**Note:** This is just an in-memory cache. The real note data lives in the Rust SQLite database. When the page loads, `invoke('get_notes')` pulls them from Rust.

---

### 2B. Rust Core Files

#### `src-tauri/src/main.rs`

```rust
// Prevents extra console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    note_taker_lib::run()
}
```

**What it does:**
- Entry point of the program
- Calls `run()` from `lib.rs` (the `_lib` suffix avoids a naming conflict on Windows)

**Rust concepts:**
- `#![cfg_attr(condition, attribute)]` — only apply an attribute if the condition is true
- `not(debug_assertions)` — true when building in release mode (`--release`)
- `windows_subsystem = "windows"` — tells Windows "don't open a console window"
- **Bottom line:** In release builds, no console window appears. In debug builds, you get a console for log messages.

---

#### `src-tauri/src/lib.rs` — THE most important file

This is the **brain** of the app. ~362 lines. Let me explain every major section.

```rust
mod api;       // include api.rs
mod db;        // include db.rs
mod state;     // include state.rs

use std::process::{Child, Command};  // import types for managing child processes
use std::sync::Mutex;                // import Mutex for thread-safe shared state
use tauri::Manager;                  // import Tauri's state management API
use uuid::Uuid;                      // import UUID generator
```

**Rust concepts for beginners:**
- `mod` — like `#include` in C or `import module` in Python. It tells Rust to include another file.
- `use` — brings a name into scope so you can use it without the full path
- `std::sync::Mutex` — a "mutual exclusion" lock. Only one thread can access the data at a time. We use it because the frontend and backend run on different threads.

##### AppError (lines 23-42)

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Db(#[from] rusqlite::Error),     // Database error
    #[error("{0}")]
    Http(String),                     // HTTP request failed
    #[error("{0}")]
    Api(String),                      // Backend returned an error
    #[error("Not authenticated")]
    NotAuthenticated,                 // User not logged in
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
```

**What it does:** Defines all the ways things can go wrong.
- We have 4 error types: DB errors, HTTP errors, API errors, and "not logged in"
- `impl Serialize` makes errors convertible to JSON so the frontend can read them

**Rust concepts:**
- `enum` — a type that can be one of several variants. Like a union or tagged union.
- `#[derive(Debug)]` — automatically adds debugging print support
- `#[error("{0}")]` — from the `thiserror` crate. `{0}` means "display the inner error's message"
- `#[from]` — allows automatic conversion: if a function returns `rusqlite::Error`, it auto-converts to `AppError::Db`
- `impl Serialize for AppError` — implement the `Serialize` trait so this type can be turned into JSON
- `&self` — a reference to the current instance (like `this` in other languages)
- `Result<S::Ok, S::Error>` — generic return type with two type parameters

##### Data Types (lines 44-66)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub sync_status: String,    // "pending" | "synced" | "deleted"
}
```

**What it does:** Defines the shape of a Note when sending it to the frontend. Each field is a string (not a number or date type) for simplicity.

**Rust concepts:**
- `struct` — a type with named fields (like a class with only data, no methods)
- `#[derive(Serialize, Deserialize)]` — auto-generate JSON conversion code
- `Clone` — lets you make copies with `.clone()`
- `pub` — makes fields accessible outside this file

```rust
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
```

**What it does:** Converts a `db::Note` (from the database module) into a `NoteResponse` (for sending to the frontend). Right now they're identical, but separating them lets you change one without breaking the other.

```rust
fn get_user_id(state: &AppState) -> Result<String, AppError> {
    let guard = state.user.lock().unwrap();     // 1. Lock the mutex
    guard                                         // 2. Access the inner value
        .clone()                                  // 3. Copy it (so we don't hold the lock)
        .map(|u| u.id)                            // 4. Extract just the ID
        .ok_or(AppError::NotAuthenticated)        // 5. If None, return error
}
```

**What it does:** Gets the current user's ID from the shared app state. If no one is logged in, returns a `NotAuthenticated` error.

**Rust concepts explained step by step:**
- `state.user` is a `Mutex<Option<UserInfo>>` — thread-safe, possibly-empty box
- `.lock()` — acquire the lock (blocks if another thread holds it)
- `.unwrap()` — if the lock is "poisoned" (another thread panicked while holding it), crash. In practice this never happens.
- `.clone()` — make an owned copy. `guard` is a temporary reference, so we need to copy the value before we can use it outside.
- `.map(|u| u.id)` — if `Option` is `Some(user)`, extract `user.id`. If `None`, do nothing.
- `.ok_or(AppError::NotAuthenticated)` — if `Option` is `None`, convert to `Err`. If `Some`, wrap in `Ok`.

##### try_start_backend (lines 79-155)

```rust
fn try_start_backend(jwt_secret: &str, db_url: &str, app: &tauri::App) -> Option<Child> {
    let cmd = |exe: &str| -> Command {
        let mut c = Command::new(exe);     // Create a new OS command
        c.env("JWT_SECRET", jwt_secret);   // Set env vars for the child process
        c.env("DATABASE_URL", db_url);
        #[cfg(windows)]                    // Windows-only code
        c.creation_flags(CREATE_NO_WINDOW); // Hide the console window!
        c
    };
    // ... tries multiple ways to start the backend ...
    // 1. Try uvicorn (dev mode)
    // 2. Try bundled backend.exe (production)
    // 3. Try local backend.exe (dev)
    // 4. Try uvicorn again (release fallback)
}
```

**What it does:** Tries to start the Python backend as a child process. It tries 4 strategies in order.

**Rust concepts:**
- `fn name(param: Type) -> ReturnType` — function declaration with types
- `&str` — a borrowed string reference (doesn't own the data, just reads it)
- `Option<Child>` — either `Some(child_process)` or `None` (couldn't start)
- `|exe: &str| -> Command { ... }` — a closure (inline anonymous function)
- `#[cfg(windows)]` — this line only compiles on Windows. On Linux/macOS, it's ignored.
- `creation_flags(0x08000000)` — Windows API constant `CREATE_NO_WINDOW`. Makes the child process run silently in the background.

##### Commands (lines 159-286)

Every function with `#[tauri::command]` can be called from the frontend via `invoke()`.

```rust
#[tauri::command]
async fn login(
    state: tauri::State<'_, AppState>,     // Shared state (injected by Tauri)
    username: String,                        // From frontend invoke()
    password: String,                        // From frontend invoke()
) -> Result<AuthResponse, AppError> {
    let resp = api::login(&state.backend_url, &username, &password).await?;
    //                                          ^ wait for HTTP response   ^ if error, return it early
    
    *state.jwt.lock().unwrap() = Some(resp.access_token.clone());  // Save JWT
    *state.user.lock().unwrap() = Some(UserInfo {                   // Save user
        id: resp.user.id.clone(),
        username: resp.user.username.clone(),
    });
    Ok(resp)  // Send response back to frontend
}
```

**Rust concepts:**
- `#[tauri::command]` — a "macro" that wraps the function so Tauri can call it from the frontend
- `async fn` — an asynchronous function (doesn't block while waiting for HTTP)
- `tauri::State<'_, AppState>` — a special parameter that Tauri injects automatically (the shared state we registered with `app.manage()`)
- `.await?` — wait for the async operation to complete. `?` means "if it returns an error, return that error from this function immediately"
- `*state.jwt.lock().unwrap() = Some(...)` — lock the mutex, then assign a new value
- `Ok(resp)` — wrap the response in `Ok` to signal success

##### Entry point (lines 302-362)

```rust
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())  // Add the "open file" plugin
        .setup(|app| {
            // Runs once when the app starts
            let db_path = app_dir.join("notes.db");
            db::init_db(&db_path_str)?;  // Create tables if needed

            let child = try_start_backend(&jwt_secret, &backend_db, app);
            app.manage(BackendProcess(Mutex::new(child)));  // Track the backend process
            app.manage(AppState { ... });                     // Register shared state
            Ok(())
        })
        .on_window_event(|window, event| {  // When window closes...
            kill_backend(window.app_handle());  // Kill the Python backend
        })
        .invoke_handler(tauri::generate_handler![  // Register all 9 commands
            register, login, get_auth_state, logout,
            get_notes, create_note, update_note, delete_note, sync_notes,
        ]);

    builder.build(...).run(...);  // Actually start the app
}
```

**What it does:** The `run()` function is the app's main entry point called from `main.rs`.

**Rust concepts:**
- `.setup(|app| { ... })` — register a callback that runs on startup
- `.on_window_event(|window, event| { ... })` — register a callback for window events (close, resize, etc.)
- `app.manage(AppState { ... })` — make this data available to all `#[tauri::command]` functions. They access it via `tauri::State<'_, AppState>`.
- `tauri::generate_handler![...]` — macro that tells Tauri which functions are callable from the frontend. Each name here must match a `#[tauri::command]` function.

---

#### `src-tauri/src/db.rs`

**What it does:** All local database operations. This is the "chef's notebook."

```rust
use rusqlite::{params, Connection};  // SQLite library

pub struct Note {
    pub id: String,
    pub user_id: String,        // ← Added for multi-user isolation
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub sync_status: String,    // "pending" → "synced" or "deleted"
}
```

**Functions (9 total):**

| Function | SQL it runs | What it does |
|----------|-----------|-------------|
| `init_db` | `CREATE TABLE IF NOT EXISTS notes (...)` | Creates DB + table on first run |
| `get_notes` | `SELECT ... WHERE user_id=? AND sync_status!='deleted'` | Get all notes for a user |
| `create_note` | `INSERT INTO notes (...)` | Add a new note |
| `update_note` | `UPDATE notes SET ... WHERE id=? AND user_id=?` | Save changes to a note |
| `delete_note` | `UPDATE notes SET sync_status='deleted' WHERE ...` | Soft-delete a note |
| `get_pending_changes` | `SELECT ... WHERE user_id=? AND sync_status IN ('pending','deleted')` | Get changes waiting to sync |
| `upsert_note` | `INSERT ... ON CONFLICT(id) DO UPDATE ...` | Insert or update (from sync) |
| `mark_synced` | `UPDATE ... SET sync_status='synced' + DELETE deleted` | Mark all as synced |
| `clear_user_notes` | `DELETE FROM notes WHERE user_id=?` | Remove all user's notes (on logout) |

**Rust concepts:**
- `params!(val1, val2)` — a macro for safe SQL parameter binding (prevents SQL injection)
- `conn.execute(sql, params)` — run SQL that doesn't return rows (INSERT, UPDATE, DELETE)
- `conn.prepare(sql)?.query_map(params, |row| { ... })` — run a query and map each row to a struct
- `query_map(params, |row| { Ok(Note { ... }) })` — `|row|` is a closure that converts a DB row to a Note
- `stmt.query_row(params, |row| { ... })` — for queries that return exactly one row
- `?` — the "try" operator: if the operation returns an error, return it from the current function

---

#### `src-tauri/src/api.rs`

**What it does:** Makes HTTP requests to the Python backend. This is the "phone" that calls the cloud kitchen.

```rust
pub async fn wait_for_backend(base_url: &str) -> Result<(), AppError> {
    // Retry up to 15 times with increasing delays
    // Health check: GET http://127.0.0.1:8000/api/health
    // Returns Ok when backend is ready
}

fn client_with_timeout() -> Result<reqwest::Client, AppError> {
    // Creates an HTTP client with 10-second timeout
}

pub async fn register(base_url, username, password) -> Result<AuthResponse, AppError> {
    // POST http://127.0.0.1:8000/api/auth/register
    // {"username": "alice", "password": "secret123"}
}

pub async fn login(base_url, username, password) -> Result<AuthResponse, AppError> {
    // POST http://127.0.0.1:8000/api/auth/login
}

pub async fn sync(base_url, token, changes) -> Result<Vec<NoteResponse>, AppError> {
    // POST http://127.0.0.1:8000/api/notes/sync
    // Header: Authorization: Bearer <token>
    // Body: {"changes": [{"id": "...", "title": "...", ...}]}
}
```

**Rust concepts:**
- `async` — this function runs asynchronously (other code can run while waiting for the HTTP response)
- `reqwest::Client::builder().timeout(...).build()` — the builder pattern. Create a config object, set properties, then build the final object.
- `.send().await` — send the HTTP request and wait for the response
- `.json::<SomeType>().await` — parse the response body as JSON into the specified Rust type
- `resp.status().is_success()` — check if HTTP status is 200-299

---

#### `src-tauri/src/state.rs`

```rust
pub struct BackendProcess(pub Mutex<Option<Child>>);
// Holds the child process handle for the Python backend

impl Drop for BackendProcess {
    fn drop(&mut self) {
        // When this struct is destroyed, kill the child process
        if let Some(ref mut child) = *self.0.lock().unwrap() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub struct AppState {
    pub jwt: Mutex<Option<String>>,       // JWT token (None = not logged in)
    pub user: Mutex<Option<UserInfo>>,    // Current user (None = not logged in)
    pub db_path: String,                   // Path to local notes.db
    pub backend_url: String,               // http://127.0.0.1:8000
}
```

**What it does:** Defines the shared state that lives for the entire app lifetime.

**Rust concepts:**
- `pub struct BackendProcess(pub Mutex<Option<Child>>)` — a "tuple struct" with a single field. Like a wrapper type.
- `impl Drop for BackendProcess` — implement the `Drop` trait. The code in `drop()` runs automatically when the struct is destroyed.
- `if let Some(ref mut child) = *guard` — pattern matching: `if let` destructures the Option, `ref mut` gives a mutable reference to the inner value.
- `let _ = child.kill()` — call kill, but ignore the result (`_` discards it).
- `Mutex<Option<...>>` — a thread-safe container for an optional value. The `Mutex` ensures only one thread accesses it at a time.

---

### 2C. Python Backend Files

(Not Rust, but included for completeness.)

| File | What it does |
|------|-------------|
| `backend/main.py` | Creates the FastAPI app, adds CORS, routes, and a `/api/health` endpoint |
| `backend/app/config.py` | Reads `DATABASE_URL` and `JWT_SECRET` from environment, with defaults |
| `backend/app/database.py` | Sets up SQLAlchemy engine to talk to SQLite |
| `backend/app/models.py` | Defines `User` and `Note` as database tables |
| `backend/app/auth.py` | Hashes passwords (bcrypt), creates and verifies JWT tokens |
| `backend/app/schemas.py` | Validates incoming requests and formats responses (Pydantic) |
| `backend/app/routers/auth.py` | Register + Login endpoints |
| `backend/app/routers/notes.py` | Notes CRUD + sync endpoint (all filter by user_id) |

---

## 3. The Mapping Table

Every `invoke()` call from the frontend, traced end-to-end.

### Login

```
Frontend                              Rust                      Backend
────────                              ────                      ───────
invoke('login', {
  username: "alice",        ───→     lib.rs: login()
  password: "secret123"              │
}                                    ├→ api::login()
                                     │   ├→ wait_for_backend()   → GET  /api/health
                                     │   └→ POST /api/auth/login → returns JWT + user
                                     │
                                     ├→ Store JWT in AppState
                                     └→ Send response back
                                              │
                ←── { access_token, user } ←──┘
```

### Register

```
invoke('register', {         ───→     lib.rs: register()
  username, password                   │
})                                     ├→ api::register()
                                       │   └→ POST /api/auth/register
                                       │
                                       ├→ Store JWT + user
                                       └→ Send response back
```

### Get Notes

```
invoke('get_notes')          ───→     lib.rs: get_notes()
                                       │
                                       ├→ get_user_id()  → "550e8400-..."
                                       ├→ db::get_notes(uid)
                                       │   └→ SELECT FROM notes WHERE user_id = ?
                                       │
                                       └→ Send note list back
                                              │
                ←── [Note, Note, ...] ←───────┘
```

**No backend contact.** This is purely local SQLite.

### Create Note

```
invoke('create_note', {      ───→     lib.rs: create_note()
  title: "Hello",                      │
  content: "World"                     ├→ get_user_id()
})                                     ├→ db::create_note(uid, id, title, content)
                                       │   └→ INSERT INTO notes (id, user_id, ...)
                                       │
                                       └→ Send created note back
```

### Update Note

```
invoke('update_note', {      ───→     lib.rs: update_note()
  id: "...",                           ├→ get_user_id()
  title: "...",                        ├→ db::update_note(uid, id, title, content)
  content: "..."                       │   └→ UPDATE notes WHERE id=? AND user_id=?
})                                     │
                                       └→ Send updated note back
```

### Delete Note

```
invoke('delete_note', {      ───→     lib.rs: delete_note()
  id: "..."                            ├→ get_user_id()
})                                     ├→ db::delete_note(uid, id)
                                       │   └→ UPDATE SET sync_status='deleted' WHERE ...
                                       │
                                       └→ Send Ok
```

### Sync Notes

This is the most complex one.

```
invoke('sync_notes')         ───→     lib.rs: sync_notes()
                                       │
                                       ├→ get_user_id() + get JWT from AppState
                                       │
                                       ├→ db::get_pending_changes(uid)
                                       │   └→ SELECT WHERE user_id=? AND (pending OR deleted)
                                       │
                                       ├→ api::sync(token, changes)
                                       │   └→ POST /api/notes/sync
                                       │      Header: Authorization: Bearer <JWT>
                                       │      Body: { changes: [...] }
                                       │      Response: { notes: [...] } (full list)
                                       │
                                       ├→ For each returned note:
                                       │   db::upsert_note(note, uid, "synced")
                                       │   └→ INSERT OR UPDATE notes WHERE id = ?
                                       │
                                       ├→ db::mark_synced(uid)
                                       │   ├→ UPDATE SET sync_status='synced' WHERE pending
                                       │   └→ DELETE WHERE deleted
                                       │
                                       └→ Send full note list back
```

### Logout

```
auth.logout() in Svelte     ───→     (frontend only — clears localStorage)

(can also call invoke('logout')):
invoke('logout')             ───→     lib.rs: logout()
                                       ├→ db::clear_user_notes(uid)
                                       │   └→ DELETE FROM notes WHERE user_id = ?
                                       ├→ Clear JWT + user from AppState
                                       └→ Send Ok
```

---

## 4. Flow Diagrams

### Startup Flow

```
App launches
    │
    ├→ main.rs calls lib.rs::run()
    │
    ├→ Tauri Builder setup:
    │   ├→ Get AppData path (C:\Users\Alice\AppData\Roaming\com.notetaker.desktop\)
    │   ├→ init_db() → CREATE TABLE IF NOT EXISTS notes
    │   ├→ Try to start backend:
    │   │   ├── Dev mode? → spawn "uvicorn main:app"
    │   │   └── Release mode? → spawn backend.exe from resources
    │   │       └── [Windows only] hidden console (CREATE_NO_WINDOW)
    │   └→ Register AppState (JWT=None, User=None, db_path, backend_url)
    │
    ├→ Tauri opens the webview window
    │
    └→ Frontend loads login page
        └→ User types credentials and clicks Login
```

### Create Note Flow (Offline)

```
User types in editor
    │
    ├→ Svelte detects oninput event
    ├→ Calls saveNote()
    ├→ invoke('create_note', { title, content })
    │
    ├→ Rust lib.rs::create_note()
    │   ├→ get_user_id() → "550e8400-..."
    │   ├→ Generate UUID for new note
    │   ├→ Generate ISO timestamp
    │   └→ db::create_note() → INSERT INTO notes
    │       ├→ id = UUID
    │       ├→ user_id = "550e8400-..."
    │       ├→ sync_status = "pending"  (needs sync!)
    │       └→ created_at = updated_at = now
    │
    └→ Returns Note to frontend
        └→ Svelte adds it to the list with a ● dot (unsynced)
```

### Sync Flow

```
User clicks Sync (↻) button
    │
    ├→ invoke('sync_notes')
    │
    ├→ Rust: collect pending changes
    │   ├→ db::get_pending_changes(uid)
    │   └→ Returns: [{id, title, content, deleted: false}, ...]
    │
    ├→ Rust: send to backend
    │   ├→ api::sync(token, changes)
    │   └→ POST /api/notes/sync
    │       ├→ Backend finds user by JWT
    │       ├→ For each change:
    │       │   ├── deleted? → DELETE FROM notes
    │       │   └── not deleted? → INSERT OR UPDATE notes
    │       └→ Returns full note list for this user
    │
    ├→ Rust: merge results
    │   ├→ For each returned note:
    │   │   └→ db::upsert_note() → INSERT OR REPLACE
    │   ├→ db::mark_synced() → pending→synced, hard delete
    │   └→ db::get_notes() → return full list
    │
    └→ Frontend updates UI
        └→ ● dots disappear (notes now synced)
```

### Logout Flow

```
User clicks Logout (→) button
    │
    ├── Option A: Svelte-only logout
    │   ├→ auth.logout()
    │   ├→ localStorage.removeItem('auth')
    │   └→ goto('/login')
    │
    └── Option B: invoke('logout') — not currently called from UI
        ├→ Rust: db::clear_user_notes(uid)  → DELETE all user's local notes
        ├→ Rust: Clear JWT + User from AppState
        └→ Frontend navigates to login page
```

---

## 5. Rust Syntax for Beginners

If you're new to Rust, here's every syntax pattern used in this project with plain-English explanations.

### Basic Syntax

| Code | What it means |
|------|--------------|
| `fn hello(name: &str) -> String` | A function named `hello` that takes a borrowed string `name` and returns an owned `String` |
| `let x = 5;` | Declare an immutable variable `x` and set it to 5 (cannot change later) |
| `let mut x = 5;` | Declare a **mutable** variable (can change later) |
| `x = 6;` | Assign a new value to a mutable variable |
| `&str` | A **string slice** — like a read-only view of a string. Think: "borrowed text" |
| `String` | An **owned** string — the program owns the memory and can modify it |
| `struct Point { x: i32, y: i32 }` | Define a struct with two integer fields |
| `Point { x: 1, y: 2 }` | Create an instance of the struct |
| `let p = Point { x: 1, y: 2 };` | Create and bind to variable |
| `p.x` | Access the `x` field |

### Ownership & Borrowing

| Code | What it means |
|------|--------------|
| `fn take_ownership(s: String)` | The function **takes ownership** of `s`. The caller can't use `s` anymore. |
| `fn borrow(s: &str)` | The function **borrows** `s`. The caller still owns it. |
| `fn modify(s: &mut String)` | The function **mutably borrows** `s`. Can change it. Caller gets it back. |
| `let s2 = s1.clone();` | **Deep copy** `s1` into `s2`. Both can be used independently. |

### Error Handling

| Code | What it means |
|------|--------------|
| `fn foo() -> Result<i32, String>` | Returns either `Ok(value)` or `Err("message")` |
| `Ok(42)` | Success! Contains value 42 |
| `Err("oops".to_string())` | Failure! Contains error message |
| `foo()?` | If `foo()` returns `Err`, return that error from **this** function immediately. If `Ok`, unwrap the value. |
| `match result { Ok(v) => ..., Err(e) => ... }` | Pattern matching — handle both cases |
| `result.map_err(\|e\| AppError::Http(e))` | Convert one error type to another |

### Option (Nullable Values)

| Code | What it means |
|------|--------------|
| `Option<String>` | A value that is either `Some(String)` or `None` |
| `let x: Option<String> = Some("hi".into());` | A value that exists |
| `let x: Option<String> = None;` | No value (like null, but type-safe) |
| `x.unwrap()` | Get the value, or **crash** if None |
| `x.unwrap_or(default)` | Get the value, or use `default` if None |
| `x.ok_or(AppError::NotAuthenticated)?` | Convert None to an error |
| `if let Some(val) = x { ... }` | Only run the block if x has a value |

### Mutex (Thread Safety)

| Code | What it means |
|------|--------------|
| `Mutex<Option<String>>` | A thread-safe box for an optional string |
| `state.lock()` | **Lock** the mutex (wait if another thread holds it). Returns a guard. |
| `state.lock().unwrap()` | Lock the mutex. `.unwrap()` crashes if the lock is "poisoned" (very rare). |
| `*guard = Some(new_val)` | Assign a new value through the guard |

**Why we need Mutex:** The frontend runs on one thread, the Rust commands might run on different threads. Without a Mutex, two threads could try to read/write the same data at the same time, causing crashes.

### Traits

| Code | What it means |
|------|--------------|
| `#[derive(Debug)]` | Auto-add `Debug` trait (can print with `{:?}`) |
| `#[derive(Clone)]` | Auto-add `Clone` trait (can call `.clone()`) |
| `#[derive(Serialize, Deserialize)]` | Auto-add JSON conversion (from `serde` crate) |
| `impl Drop for BackendProcess` | Implement the `Drop` trait — code runs when value is destroyed |
| `impl Serialize for AppError` | Manually implement `Serialize` trait |

### Macros & Attributes

| Code | What it means |
|------|--------------|
| `#[tauri::command]` | Marks function as callable from frontend via `invoke()` |
| `#[cfg(windows)]` | Only compile this line on Windows |
| `#[cfg(debug_assertions)]` | Only compile this line in debug builds |
| `format!("hello {}", name)` | String formatting (like f-strings in Python) |
| `eprintln!("Error: {}", msg)` | Print to stderr (error output) |
| `params![val1, val2]` | Safe SQL parameter binding (prevents SQL injection) |

### Common Patterns in This Project

| Pattern | Where used | What it does |
|---------|-----------|-------------|
| `let mut guard = state.x.lock().unwrap();` | Every command in `lib.rs` | Lock shared state before accessing |
| `*guard = Some(new_value);` | `login()`, `register()` | Update shared state |
| `let uid = get_user_id(&state)?;` | All CRUD commands | Get current user or return error |
| `db::some_fn(&state.db_path, &uid, ...)?;` | All CRUD commands | Run DB operation, propagate error |
| `api::some_call(...).await?;` | `login()`, `register()`, `sync_notes()` | Make HTTP request, propagate error |
| `match Command::new(...).spawn() { Ok(c) => ..., Err(e) => ... }` | `try_start_backend()` | Try to start a process, handle failure gracefully |

### Flow of a Typical Command

```
Frontend                  Rust                            Result Type
────────                  ────                            ───────────
invoke('get_notes')  →    fn get_notes(...) -> Result<Vec<NoteResponse>, AppError>
                              │
                              ├→ get_user_id(&state)?
                              │   Returns: Result<String, AppError>
                              │            Ok("550e...") or Err(NotAuthenticated)
                              │   The ? means: if Err, return it to frontend
                              │
                              ├→ db::get_notes(&state.db_path, &uid)?
                              │   Returns: Result<Vec<Note>, AppError>
                              │            Ok([Note, Note, ...]) or Err(Db(error))
                              │   The ? means: if Err, return it to frontend
                              │
                              └→ Ok(notes...)  
                                 Returns: Result<Vec<NoteResponse>, AppError>
                                          success! send notes to frontend
```

---

## 6. What Lives Where

### Data Storage Map

```
┌─────────────────────────────────────────────────────────┐
│                   AppData Directory                      │
│  C:\Users\<username>\AppData\Roaming\com.notetaker.desktop│
├─────────────────────────────────────────────────────────┤
│                                                          │
│  notes.db (Rust / rusqlite)                              │
│  ├── Contains: all notes for the logged-in user          │
│  ├── Filtered by: user_id column                         │
│  ├── Per-user: YES (because AppData is per-user)         │
│  └── Created: on first app launch                        │
│                                                          │
│  note_taker.db (Python / aiosqlite)                      │
│  ├── Contains: users table + notes table (for sync)      │
│  ├── Filtered by: user_id FK in notes table              │
│  ├── Per-user: YES (DATABASE_URL env var → AppData)      │
│  └── Created: on first backend request                   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Which File Talks to What

| File | Talks to | How |
|------|----------|-----|
| `*.svelte` | Rust (lib.rs) | `invoke('command')` — Tauri IPC |
| `lib.rs` | Local SQLite | `db::get_notes()` etc. — direct function calls |
| `lib.rs` | Python Backend | `api::login()` etc. — HTTP via `reqwest` |
| `lib.rs` | OS (backend.exe) | `Command::new("uvicorn/backend.exe")` — child process |
| `backend/routers/*.py` | Remote SQLite | SQLAlchemy ORM — `async with async_session()` |
| `auth.ts` | localStorage | `localStorage.setItem('auth', ...)` — browser storage |
| `notes.ts` | In-memory | Svelte store — reactive array |

---

## 7. Startup Sequence

### What happens when the user double-clicks note_taker.exe:

```
Step 1: Windows launches note_taker.exe (the Rust/Tauri binary)
        │
Step 2: main.rs runs, calls note_taker_lib::run()
        │
Step 3: lib.rs::run() starts Tauri Builder
        │
Step 4: builder.setup() callback fires:
        │
        ├── Get AppData directory
        │   C:\Users\Alice\AppData\Roaming\com.notetaker.desktop\
        │
        ├── Create AppData directory if it doesn't exist
        │
        ├── Initialize local SQLite
        │   ├── File: AppData\notes.db
        │   └── CREATE TABLE IF NOT EXISTS notes (id, user_id, ...)
        │
        ├── Build backend database URL
        │   sqlite+aiosqlite:///C:/Users/.../com.notetaker.desktop/note_taker.db
        │
        ├── Check JWT_SECRET environment variable
        │   (not set → backend will auto-generate its own)
        │
        ├── Try to start the Python backend:
        │   ├── Strategy 1: uvicorn (dev mode, hot-reload)
        │   ├── Strategy 2: bundled backend.exe from resources (production)
        │   ├── Strategy 3: backend.exe from relative path (dev)
        │   └── Strategy 4: uvicorn fallback (release, if exe not found)
        │
        │   On Windows, the backend starts with CREATE_NO_WINDOW
        │   → no console window (runs silently in background)
        │
        ├── Manage shared state:
        │   ├── BackendProcess → track child process handle
        │   └── AppState → JWT: None, User: None, db_path, backend_url
        │
        └── Register all 9 commands with invoke_handler
            register, login, get_auth_state, logout,
            get_notes, create_note, update_note, delete_note, sync_notes
        │
Step 5: Tauri opens the webview window
        │
Step 6: SvelteKit loads login page (user sees login form)
        │
Step 7: User types credentials, clicks Login
        │
Step 8: invoke('login', { username, password })
        │
        ├── Rust calls wait_for_backend() — polls /api/health up to 15 times
        │   (The backend might still be starting up, so we wait)
        │
        ├── Rust calls api::login() → POST /api/auth/login
        │   Backend creates JWT token, returns it
        │
        ├── Rust stores JWT + UserInfo in AppState
        │
        └── Frontend navigates to /notes
        │
Step 9: Notes page loads, calls invoke('get_notes')
        └── Returns empty list (first time user) or saved notes
```

### What happens when the user closes the window:

```
Step 1: Window close event fires
        │
Step 2: on_window_event → kill_backend()
        │           └── child.kill() + child.wait() → backend.exe is terminated
        │
Step 3: RunEvent::Exit fires
        │           └── kill_backend() again (just in case)
        │
Step 4: BackendProcess Drop impl fires
        │           └── (guard is already None, so nothing happens)
        │
Step 5: Process exits cleanly
        └── No orphan processes, no leftover backend.exe
```

---

*ARCHITECTURE.md — Generated by opencode on 2026-05-30.*
