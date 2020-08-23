use std::env;

use sqlx::{
    Error as SqlError,
    prelude::*,
    sqlite::SqlitePool,
};

pub async fn database_connect() -> Result<SqlitePool, SqlError> {
    let path = env::var("DATABASE_URL").expect("No DATABASE_URL found");
    SqlitePool::new(&*path).await
}

pub async fn initialise_database_tables<C: Executor>(db: &mut C) -> Result<u64, SqlError> {
    db.execute("
    CREATE TABLE IF NOT EXISTS boxnovel(
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    title TEXT NOT NULL,
    novel TEXT NOT NULL,
    current TEXT NOT NULL
    )").await
}