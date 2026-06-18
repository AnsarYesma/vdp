use sqlx::{FromRow, SqlitePool};

#[derive(FromRow)]
pub struct Message {
    pub id: i64,
    pub message: String,
    pub proof_hex: String,
    pub t: i64,
    pub is_demo: bool,
    pub verified: bool,
    pub created_at: String,
}

pub async fn init(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            message    TEXT    NOT NULL,
            proof_hex  TEXT    NOT NULL,
            t          INTEGER NOT NULL,
            is_demo    BOOLEAN NOT NULL DEFAULT 0,
            verified   BOOLEAN NOT NULL DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool)
    .await
    .expect("failed to create messages table");
}

pub async fn count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM messages")
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

pub async fn insert(
    pool: &SqlitePool,
    message: &str,
    proof_hex: &str,
    t: u64,
    is_demo: bool,
    verified: bool,
) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO messages (message, proof_hex, t, is_demo, verified) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(message)
    .bind(proof_hex)
    .bind(t as i64)
    .bind(is_demo)
    .bind(verified)
    .fetch_one(pool)
    .await
    .expect("failed to insert message")
}

pub async fn list(pool: &SqlitePool) -> Vec<Message> {
    sqlx::query_as::<_, Message>(
        "SELECT id, message, proof_hex, t, is_demo, verified, created_at FROM messages ORDER BY t DESC, id ASC",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

pub async fn get(pool: &SqlitePool, id: i64) -> Option<Message> {
    sqlx::query_as::<_, Message>(
        "SELECT id, message, proof_hex, t, is_demo, verified, created_at FROM messages WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None)
}
