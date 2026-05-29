use rusqlite::{params, Connection};

use crate::AppError;

pub struct Note {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub sync_status: String,
}

pub fn init_db(db_path: &str) -> Result<(), AppError> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS notes (
            id         TEXT PRIMARY KEY,
            user_id    TEXT NOT NULL DEFAULT '',
            title      TEXT NOT NULL DEFAULT '',
            content    TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            sync_status TEXT NOT NULL DEFAULT 'pending'
        );",
    )?;
    Ok(())
}

pub fn get_notes(db_path: &str, user_id: &str) -> Result<Vec<Note>, AppError> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, title, content, created_at, updated_at, sync_status
         FROM notes
         WHERE user_id = ?1 AND sync_status != 'deleted'
         ORDER BY updated_at DESC",
    )?;
    let notes = stmt
        .query_map(params![user_id], |row| {
            Ok(Note {
                id: row.get(0)?,
                user_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                sync_status: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

pub fn create_note(
    db_path: &str,
    user_id: &str,
    id: &str,
    title: &str,
    content: &str,
    created_at: &str,
    updated_at: &str,
) -> Result<Note, AppError> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT INTO notes (id, user_id, title, content, created_at, updated_at, sync_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')",
        params![id, user_id, title, content, created_at, updated_at],
    )?;
    Ok(Note {
        id: id.to_string(),
        user_id: user_id.to_string(),
        title: title.to_string(),
        content: content.to_string(),
        created_at: created_at.to_string(),
        updated_at: updated_at.to_string(),
        sync_status: "pending".to_string(),
    })
}

pub fn update_note(
    db_path: &str,
    user_id: &str,
    id: &str,
    title: &str,
    content: &str,
    updated_at: &str,
) -> Result<Note, AppError> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3, sync_status = 'pending'
         WHERE id = ?4 AND user_id = ?5",
        params![title, content, updated_at, id, user_id],
    )?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, title, content, created_at, updated_at, sync_status FROM notes WHERE id = ?1 AND user_id = ?2",
    )?;
    let note = stmt.query_row(params![id, user_id], |row| {
        Ok(Note {
            id: row.get(0)?,
            user_id: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            sync_status: row.get(6)?,
        })
    })?;
    Ok(note)
}

pub fn delete_note(db_path: &str, user_id: &str, id: &str) -> Result<(), AppError> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "UPDATE notes SET sync_status = 'deleted' WHERE id = ?1 AND user_id = ?2",
        params![id, user_id],
    )?;
    Ok(())
}

pub fn get_pending_changes(db_path: &str, user_id: &str) -> Result<Vec<(Note, bool)>, AppError> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, title, content, created_at, updated_at, sync_status
         FROM notes WHERE user_id = ?1 AND sync_status IN ('pending', 'deleted')",
    )?;
    let changes = stmt
        .query_map(params![user_id], |row| {
            let sync_status: String = row.get(6)?;
            Ok((
                Note {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    sync_status: sync_status.clone(),
                },
                sync_status == "deleted",
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(changes)
}

pub fn upsert_note(db_path: &str, note: &Note, user_id: &str, sync_status: &str) -> Result<(), AppError> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT INTO notes (id, user_id, title, content, created_at, updated_at, sync_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            user_id = excluded.user_id,
            title = excluded.title,
            content = excluded.content,
            created_at = excluded.created_at,
            updated_at = excluded.updated_at,
            sync_status = excluded.sync_status",
        params![
            note.id,
            user_id,
            note.title,
            note.content,
            note.created_at,
            note.updated_at,
            sync_status
        ],
    )?;
    Ok(())
}

pub fn mark_synced(db_path: &str, user_id: &str) -> Result<(), AppError> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "UPDATE notes SET sync_status = 'synced' WHERE user_id = ?1 AND sync_status = 'pending'",
        params![user_id],
    )?;
    conn.execute(
        "DELETE FROM notes WHERE user_id = ?1 AND sync_status = 'deleted'",
        params![user_id],
    )?;
    Ok(())
}

pub fn clear_user_notes(db_path: &str, user_id: &str) -> Result<(), AppError> {
    let conn = Connection::open(db_path)?;
    conn.execute("DELETE FROM notes WHERE user_id = ?1", params![user_id])?;
    Ok(())
}

