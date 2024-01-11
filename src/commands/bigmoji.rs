use anyhow::{anyhow, Context};
use chrono::NaiveDateTime;
use serenity::all::Interaction;
use sqlx::SqlitePool;
use tracing::instrument;

use crate::helpers::get_cmd;

#[derive(sqlx::FromRow)]
pub struct BigMoji {
    pub name: String,
    pub text: String,
    pub inserted_at: NaiveDateTime,
}

pub async fn add(db: SqlitePool, interaction: &Interaction) -> anyhow::Result<String> {
    let cmd = get_cmd(interaction)?;

    let mut name = cmd.name.replace(':', "").to_lowercase();
    name.retain(|c| !c.is_whitespace());

    let text = cmd.value.as_str().ok_or(anyhow!("bigmoji text"))?;

    add_handler(db, name.as_str(), text).await
}

#[instrument]
async fn add_handler(db: SqlitePool, name: &str, text: &str) -> anyhow::Result<String> {
    if name.len() < 3 {
        return Ok("BigMoji name too short".to_string());
    }

    // Prevents recursive BigMoji
    let text = text.replace(&format!(":{}:", name), "");

    sqlx::query!(
        "INSERT INTO bigmoji (name, text) VALUES (?, ?);",
        name,
        text
    )
    .execute(&db)
    .await
    .with_context(|| "inserting bigmoji")?;

    Ok(format!("BigMoji `:{}:` added", name))
}

pub async fn remove(db: SqlitePool, interaction: &Interaction) -> anyhow::Result<String> {
    let cmd = get_cmd(interaction)?;
    let mut name = cmd.name.replace(':', "").to_lowercase();
    name.retain(|c| !c.is_whitespace());

    remove_handler(db, name.as_str()).await
}

#[instrument]
async fn remove_handler(db: SqlitePool, name: &str) -> anyhow::Result<String> {
    sqlx::query!("DELETE FROM bigmoji WHERE name = ?;", name)
        .execute(&db)
        .await
        .with_context(|| "deleting bigmoji")?;

    Ok(format!("Deleted BigMoji `:{}:`", name))
}

pub async fn get(db: SqlitePool, interaction: &Interaction) -> anyhow::Result<String> {
    let cmd = get_cmd(interaction)?;

    let mut name = cmd.name.replace(':', "").to_lowercase();
    name.retain(|c| !c.is_whitespace());

    get_handler(db, name.as_str()).await
}

#[instrument]
async fn get_handler(db: SqlitePool, name: &str) -> anyhow::Result<String> {
    let moji: Option<BigMoji> =
        sqlx::query_as!(BigMoji, "SELECT * FROM bigmoji WHERE name = ?;", name)
            .fetch_optional(&db)
            .await
            .with_context(|| "getting bigmoji")?;

    Ok(moji
        .map(|m| m.text)
        .unwrap_or(format!("BigMoji `:{}:` does not exist", name)))
}
