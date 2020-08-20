use sqlx::{
    prelude::*,
    sqlite::SqlitePool,
    Error as SqlError,
};

pub async fn database_connect() -> Result<SqlitePool, SqlError> {
    SqlitePool::new("sqlite::feed.db").await
}
pub async fn initialise_database_tables<C: Executor>(db: &mut C) -> Result<u64, SqlError> {
    db.execute("
    CREATE TABLE IF NOT EXISTS boxnovel(
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    novel TEXT NOT NULL,
    current TEXT NOT NULL
    )").await
}