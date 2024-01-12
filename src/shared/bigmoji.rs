use anyhow::Context;
use chrono::NaiveDateTime;
use sqlx::SqlitePool;
use tracing::instrument;

#[derive(sqlx::FromRow)]
pub struct BigMoji {
    pub name: String,
    pub text: String,
    pub inserted_at: NaiveDateTime,
}

#[instrument]
pub async fn add(db: SqlitePool, name: &str, text: &str) -> anyhow::Result<()> {
    // Prevents recursive BigMoji
    let text = text.replace(&format!(":{}:", name), "");

    sqlx::query!(
        "INSERT INTO bigmoji (name, text) VALUES (?, ?);",
        name,
        text
    )
    .execute(&db)
    .await
    .with_context(|| "inserting bigmoji")
    .map(|_| ())
}

#[instrument]
pub async fn remove(db: SqlitePool, name: &str) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM bigmoji WHERE name = ?;", name)
        .execute(&db)
        .await
        .with_context(|| "deleting bigmoji")
        .map(|_| ())
}

#[instrument]
pub async fn get_one(db: SqlitePool, name: &str) -> anyhow::Result<Option<BigMoji>> {
    sqlx::query_as!(BigMoji, "SELECT * FROM bigmoji WHERE name = ?;", name)
        .fetch_optional(&db)
        .await
        .with_context(|| "getting bigmoji")
}

#[instrument]
pub async fn get_all(db: SqlitePool) -> anyhow::Result<Vec<BigMoji>> {
    sqlx::query_as!(BigMoji, "SELECT * FROM bigmoji;")
        .fetch_all(&db)
        .await
        .with_context(|| "get bigmoji")
}
