# Note Taker — Software Development Life Cycle (SDLC)

> **Version:** 0.1.0
> **Date:** 2026-05-30
> **License:** MIT
> **Author:** opencode

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Architecture](#2-architecture)
3. [Tech Stack](#3-tech-stack)
4. [Development Timeline](#4-development-timeline)
5. [Phase 1 — Initial Scaffold](#5-phase-1--initial-scaffold)
6. [Phase 2 — Backend Implementation](#6-phase-2--backend-implementation)
7. [Phase 3 — Offline-First Local Storage](#7-phase-3--offline-first-local-storage)
8. [Phase 4 — Backend Compilation & Bundling](#8-phase-4--backend-compilation--bundling)
9. [Phase 5 — Production Packaging](#9-phase-5--production-packaging)
10. [Phase 6 — UX Hardening](#10-phase-6--ux-hardening)
11. [Phase 7 — Multi-User Isolation](#11-phase-7--multi-user-isolation)
12. [Database Schemas](#12-database-schemas)
13. [API Reference](#13-api-reference)
14. [Data Flow Diagram](#14-data-flow-diagram)
15. [Project Structure](#15-project-structure)
16. [Build & Run](#16-build--run)
17. [Known Issues & Future Work](#17-known-issues--future-work)

---

## 1. Project Overview

A **desktop note-taking application** with offline-first architecture, optional cloud sync, and multi-user support. Built with Tauri (Rust + SvelteKit) and a bundled FastAPI backend.

### Core Requirements

| # | Requirement | Status |
|---|-------------|--------|
| 1 | CRUD operations on notes | Done |
| 2 | Offline-first (local SQLite) | Done |
| 3 | Manual sync to remote backend | Done |
| 4 | User authentication (register/login) | Done |
| 5 | JWT-based session management | Done |
| 6 | Self-contained installer (no Python/Docker required) | Done |
| 7 | Multi-user isolation on shared machines | Done |
| 8 | Hidden backend process (no console window) | Done |
| 9 | Graceful backend failure handling | Done |
| 10 | Clean process lifecycle (no orphan processes) | Done |

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Tauri Desktop App                   │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  SvelteKit   │  │  Rust Core    │  │  Backend   │ │
│  │  Frontend    │◄►│  (lib.rs)     │◄►│  (exe)     │ │
│  │  (WebView)   │  │  Tauri IPC    │  │  FastAPI   │ │
│  └─────────────┘  └──────┬───────┘  └─────┬──────┘ │
│                          │                 │         │
│                    ┌─────┴──────┐   ┌─────┴──────┐  │
│                    │ notes.db   │   │note_taker  │  │
│                    │ (rusqlite) │   │   .db      │  │
│                    │ Per-User   │   │(aiosqlite) │  │
│                    └────────────┘   │ Per-User   │  │
│                                     └────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Communication Layers

| Layer | Protocol | Purpose |
|-------|----------|---------|
| Frontend ↔ Rust | Tauri IPC (`invoke`) | UI commands (CRUD, auth, sync) |
| Rust ↔ Backend | HTTP (`reqwest`) | `http://127.0.0.1:8000/api/*` |
| Rust ↔ Local DB | `rusqlite` (synchronous) | Offline note storage |
| Backend ↔ Remote DB | `aiosqlite` (async SQLAlchemy) | Cloud storage + auth |

---

## 3. Tech Stack

### Frontend
| Technology | Version | Purpose |
|------------|---------|---------|
| SvelteKit | ^2.9.0 | App framework (SSG mode) |
| Svelte | ^5.0.0 | UI components ($state runes) |
| TypeScript | ~5.6.2 | Type safety |
| Vite | ^6.0.3 | Build tool |
| @sveltejs/adapter-static | ^3.0.6 | Static export for Tauri |

### Backend (Rust Core)
| Crate | Version | Purpose |
|-------|---------|---------|
| tauri | 2 | Desktop framework |
| rusqlite | 0.31 | Local SQLite (bundled) |
| reqwest | 0.12 | HTTP client for backend API |
| tokio | 1 (full) | Async runtime |
| uuid | 1 | Note ID generation |
| chrono | 0.4 | Timestamps |
| serde / serde_json | 1 | Serialization |
| thiserror | 1 | Error handling |

### Backend (Python API)
| Library | Version | Purpose |
|---------|---------|---------|
| FastAPI | latest | REST API framework |
| SQLAlchemy (async) | latest | ORM + async DB |
| aiosqlite | latest | Async SQLite driver |
| python-jose | latest | JWT tokens |
| bcrypt | latest | Password hashing |
| PyInstaller | latest | EXE compilation |

---

## 4. Development Timeline

| Phase | Date | Description |
|-------|------|-------------|
| 1 | 2026-05-29 | Initial scaffold (Tauri + SvelteKit) |
| 2 | 2026-05-29 | Backend API (FastAPI + auth + notes + sync) |
| 3 | 2026-05-29 | Offline-first local storage (Rust + SQLite) |
| 4 | 2026-05-29 | Backend compilation (PyInstaller) + bundling |
| 5 | 2026-05-29 | Production packaging (MSI + NSIS installers) |
| 6 | 2026-05-30 | UX hardening (hidden window, timeouts, error banner) |
| 7 | 2026-05-30 | Multi-user isolation (per-user DB paths + user_id filtering) |

---

## 5. Phase 1 — Initial Scaffold

**Goal:** Bootstrap Tauri v2 + SvelteKit project with static adapter.

### What was done
- Created Tauri project with `create-tauri-app`
- Configured SvelteKit with `adapter-static` (required for Tauri — no Node server)
- Set `prerender = true` and `ssr = false` in `+layout.ts`
- Configured dev server on `localhost:1420`
- Set up TypeScript + Svelte 5 runes

### Key decisions
- **SvelteKit static adapter** — Tauri doesn't have a Node.js runtime, so the frontend must be fully client-side rendered
- **SSR disabled** — All data comes from the Rust backend via Tauri IPC, not server-side rendering

---

## 6. Phase 2 — Backend Implementation

**Goal:** Build a complete FastAPI backend with authentication, notes CRUD, and sync.

### What was done
- Created FastAPI app with CORS middleware
- Implemented SQLAlchemy models (`User`, `Note`) with async SQLite
- Built auth system: register, login, JWT tokens, bcrypt password hashing
- Built notes API: list, create, update, delete (all user-scoped)
- Built sync endpoint: bidirectional note synchronization
- Added health check endpoint (`/api/health`)
- Made `JWT_SECRET` optional (auto-generates UUID if not set)

### API Endpoints

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/api/health` | No | Backend liveness check |
| POST | `/api/auth/register` | No | Create user account |
| POST | `/api/auth/login` | No | Authenticate user |
| GET | `/api/notes` | JWT | List user's notes |
| POST | `/api/notes` | JWT | Create a note |
| GET | `/api/notes/{id}` | JWT | Get a note |
| PUT | `/api/notes/{id}` | JWT | Update a note |
| DELETE | `/api/notes/{id}` | JWT | Delete a note |
| POST | `/api/notes/sync` | JWT | Bidirectional sync |

### Config (`backend/app/config.py`)
```python
DATABASE_URL = os.getenv("DATABASE_URL", "sqlite+aiosqlite:///./note_taker.db")
JWT_SECRET = os.getenv("JWT_SECRET") or str(uuid.uuid4())
JWT_ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 1440  # 24 hours
```

---

## 7. Phase 3 — Offline-First Local Storage

**Goal:** Notes work fully offline. Sync is manual and optional.

### What was done
- Created Rust `db.rs` module with `rusqlite`
- Implemented local CRUD operations (get, create, update, delete)
- Added `sync_status` tracking (`pending`, `synced`, `deleted`)
- Built soft-delete system (notes marked as deleted, purged after sync)
- Created Tauri IPC commands that bridge frontend ↔ local SQLite
- Sync sends pending changes to backend, merges remote responses

### Local DB Schema
```sql
CREATE TABLE notes (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL DEFAULT '',
    title       TEXT NOT NULL DEFAULT '',
    content     TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    sync_status TEXT NOT NULL DEFAULT 'pending'
);
```

### Sync Protocol
1. Collect all notes with `sync_status IN ('pending', 'deleted')`
2. Send as `SyncRequest` to `POST /api/notes/sync`
3. Backend merges changes (conflict resolution: latest `updated_at` wins)
4. Backend returns full list of user's notes
5. Local DB upserts all remote notes
6. Mark synced notes as `synced`, hard-delete `deleted` ones

---

## 8. Phase 4 — Backend Compilation & Bundling

**Goal:** Package the Python backend as a standalone `.exe` — no Python required on end-user machines.

### What was done
- Compiled `backend.exe` with PyInstaller (`--onefile`)
- Hidden imports: `uvicorn.*`, `aiosqlite`, `sqlalchemy.sql.default_comparator`
- Final exe size: ~24 MB (includes Python runtime + all deps)
- Configured Tauri to bundle `bin/backend.exe` as a resource
- Updated Rust startup logic: dev mode uses `uvicorn`, release mode uses bundled exe

### Bundle config (`tauri.conf.json`)
```json
{
  "bundle": {
    "resources": ["bin/backend.exe"]
  }
}
```

### Backend startup logic (`lib.rs`)
```
#[cfg(debug_assertions)]  →  spawn uvicorn (dev, hot-reload)
#[cfg(not(debug_assertions))]  →  spawn backend.exe from bundled resources
```

---

## 9. Phase 5 — Production Packaging

**Goal:** Build MSI and NSIS installers that include everything.

### What was done
- Ran `npm run tauri build` successfully
- Generated two installers:
  - `note_taker_0.1.0_x64_en-US.msi` (~30 MB)
  - `note_taker_0.1.0_x64-setup.exe` (~28 MB)
- Both installers include `backend.exe` as a bundled resource
- No external dependencies required (Python, Docker, PostgreSQL all eliminated)

### Installer locations
```
src-tauri/target/release/bundle/msi/note_taker_0.1.0_x64_en-US.msi
src-tauri/target/release/bundle/nsis/note_taker_0.1.0_x64-setup.exe
```

---

## 10. Phase 6 — UX Hardening

**Goal:** Fix production UX issues (console window, hanging, orphan processes).

### Problems solved

| Problem | Root Cause | Fix |
|---------|-----------|-----|
| Console window visible | `Command::spawn()` opens console on Windows | `CREATE_NO_WINDOW` flag (`0x08000000`) |
| Frontend hangs when backend dies | No timeout on reqwest calls | 10s timeout on all HTTP clients |
| No error feedback on notes page | Empty `catch {}` blocks | Red error banner with auto-dismiss |
| Backend survives app close (orphan) | No explicit cleanup | `on_window_event` + `RunEvent::Exit` handler |
| Backend port conflict | Port 8000 held by zombie process | Backend killed on window close + app exit |

### Files changed
- `src-tauri/src/lib.rs` — `CREATE_NO_WINDOW`, `kill_backend()`, `on_window_event`
- `src-tauri/src/api.rs` — `client_with_timeout()` (10s)
- `src/routes/notes/+page.svelte` — `backendErr` banner

---

## 11. Phase 7 — Multi-User Isolation

**Goal:** Different users on the same machine see different data.

### Problems found
1. Backend `DATABASE_URL` defaulted to `./note_taker.db` — shared across all Windows users
2. Local `notes.db` had no `user_id` column — all notes dumped into one flat table

### Fixes

| Layer | Before | After |
|-------|--------|-------|
| Backend DB | `sqlite+aiosqlite:///./note_taker.db` (shared) | `DATABASE_URL` env var → per-user `AppData` path |
| Local DB schema | No `user_id` column | `user_id TEXT NOT NULL DEFAULT ''` |
| Local queries | No filtering | All queries filter by `user_id` |
| Logout | Just cleared JWT | Clears user's local notes |

### Per-user paths (Windows)
```
Local notes:  C:\Users\<user>\AppData\Roaming\com.notetaker.desktop\notes.db
Backend DB:   C:\Users\<user>\AppData\Roaming\com.notetaker.desktop\note_taker.db
```

### Files changed
- `src-tauri/src/db.rs` — Added `user_id` to schema + all queries
- `src-tauri/src/lib.rs` — Pass `user_id` from `AppState` to all DB calls, pass `DATABASE_URL` to backend
- `src-tauri/src/state.rs` — No change (already had user info)

---

## 12. Database Schemas

### Local SQLite (Rust — `notes.db`)

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | UUID v4 |
| `user_id` | TEXT | Owning user's ID |
| `title` | TEXT | Note title |
| `content` | TEXT | Note body |
| `created_at` | TEXT | ISO 8601 timestamp |
| `updated_at` | TEXT | ISO 8601 timestamp |
| `sync_status` | TEXT | `pending` / `synced` / `deleted` |

### Remote SQLite (Python — `note_taker.db`)

**users table**

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID PK | Auto-generated |
| `username` | TEXT UNIQUE | Login name |
| `hashed_password` | TEXT | bcrypt hash |
| `created_at` | DATETIME | Account creation |

**notes table**

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID PK | Auto-generated |
| `user_id` | UUID FK → users.id | Owner |
| `title` | TEXT | Note title |
| `content` | TEXT | Note body |
| `created_at` | DATETIME | Creation time |
| `updated_at` | DATETIME | Last modified |

---

## 13. API Reference

### POST /api/auth/register
```json
// Request
{ "username": "alice", "password": "secret123" }

// Response 201
{
  "access_token": "eyJ...",
  "token_type": "bearer",
  "user": { "id": "550e8400...", "username": "alice", "created_at": "..." }
}
```

### POST /api/auth/login
```json
// Request
{ "username": "alice", "password": "secret123" }

// Response 200
{
  "access_token": "eyJ...",
  "token_type": "bearer",
  "user": { "id": "550e8400...", "username": "alice", "created_at": "..." }
}
```

### POST /api/notes/sync
```json
// Request (Header: Authorization: Bearer <token>)
{
  "changes": [
    { "id": "...", "title": "My Note", "content": "Hello", "updated_at": "2026-05-30T12:00:00Z", "deleted": false }
  ]
}

// Response 200
{
  "notes": [
    { "id": "...", "title": "My Note", "content": "Hello", "created_at": "...", "updated_at": "..." }
  ],
  "server_time": "2026-05-30T12:00:01Z"
}
```

---

## 14. Data Flow Diagram

### Offline CRUD
```
User → SvelteKit → invoke('create_note') → Rust db.rs → notes.db
User → SvelteKit → invoke('get_notes')   ← Rust db.rs ← notes.db
```

### Sync
```
User clicks Sync
  → invoke('sync_notes')
  → Rust collects pending changes from notes.db
  → POST /api/notes/sync (with JWT)
  → Backend merges, returns full note list
  → Rust upserts into notes.db
  → Frontend re-renders
```

### Authentication
```
User logs in
  → invoke('login', {username, password})
  → Rust → POST /api/auth/login
  → Backend verifies bcrypt, returns JWT
  → Rust stores JWT + user info in AppState
  → Frontend stores in Svelte store + localStorage
```

---

## 15. Project Structure

```
note_taker/
├── backend/
│   ├── app/
│   │   ├── __init__.py
│   │   ├── auth.py              # JWT + bcrypt
│   │   ├── config.py            # DATABASE_URL, JWT_SECRET
│   │   ├── database.py          # SQLAlchemy engine
│   │   ├── models.py            # User, Note ORM models
│   │   ├── routers/
│   │   │   ├── auth.py          # /api/auth/*
│   │   │   └── notes.py         # /api/notes/*
│   │   └── schemas.py           # Pydantic models
│   ├── main.py                  # FastAPI app entry
│   └── backend.exe              # Compiled (PyInstaller)
├── src/
│   ├── lib/stores/
│   │   ├── auth.ts              # Auth Svelte store
│   │   └── notes.ts             # Notes Svelte store
│   ├── routes/
│   │   ├── +layout.ts           # SSR off, prerender on
│   │   ├── +page.svelte         # Root redirect
│   │   ├── login/+page.svelte   # Login form
│   │   ├── register/+page.svelte# Register form
│   │   └── notes/+page.svelte   # Notes editor
│   └── app.html
├── src-tauri/
│   ├── bin/
│   │   └── backend.exe          # Bundled for production
│   ├── src/
│   │   ├── api.rs               # HTTP client (reqwest)
│   │   ├── db.rs                # Local SQLite (rusqlite)
│   │   ├── lib.rs               # Tauri commands + setup
│   │   ├── main.rs              # Entry point
│   │   └── state.rs             # AppState, BackendProcess
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
├── package.json
├── svelte.config.js
├── vite.config.js
└── SDLC.md                      # This file
```

---

## 16. Build & Run

### Development
```bash
npm install
npm run tauri dev
```

### Production build
```bash
npm run tauri build
```

### Backend rebuild (if Python code changes)
```bash
cd backend
pyinstaller --onefile --name backend \
  --hidden-import uvicorn.logging \
  --hidden-import uvicorn.loops.auto \
  --hidden-import uvicorn.protocols.http.auto \
  --hidden-import sqlalchemy.sql.default_comparator \
  --hidden-import aiosqlite \
  --distpath . main.py
copy backend.exe ..\src-tauri\bin\
```

### Installers
```
src-tauri/target/release/bundle/msi/note_taker_0.1.0_x64_en-US.msi
src-tauri/target/release/bundle/nsis/note_taker_0.1.0_x64-setup.exe
```

### End-user requirements
- Windows (x64)
- Nothing else. No Python, Docker, PostgreSQL, or environment variables.

---

## 17. Known Issues & Future Work

### Known Issues
| # | Issue | Severity | Notes |
|---|-------|----------|-------|
| 1 | SQLite not suitable for high-concurrency multi-device sync | Medium | Fine for single-user desktop |
| 2 | JWT secret auto-generated per machine — sync across devices requires shared secret | Low | By design for security |
| 3 | No note search functionality | Low | Could add FTS5 |
| 4 | No rich text / markdown support | Low | Plain text only |
| 5 | No note categories / folders | Low | Flat list only |

### Future Work
| # | Feature | Priority | Complexity |
|---|---------|----------|-----------|
| 1 | FTS5 full-text search | High | Low |
| 2 | Note export (markdown/PDF) | Medium | Medium |
| 3 | Dark mode | Medium | Low |
| 4 | Markdown preview | Medium | Medium |
| 5 | Note pinning / favorites | Low | Low |
| 6 | Cross-device sync (shared JWT secret) | Medium | High |
| 7 | Linux / macOS builds | Medium | Medium |
| 8 | Auto-update mechanism | Low | High |
| 9 | Note encryption at rest | Low | High |
| 10 | Drag-and-drop reordering | Low | Medium |

---

*Generated by opencode — 2026-05-30*
