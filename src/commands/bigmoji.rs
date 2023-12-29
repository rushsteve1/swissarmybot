use anyhow::{anyhow, Context};
use chrono::NaiveDateTime;
use serenity::all::Interaction;
use tracing::instrument;

use crate::{helpers::get_cmd, DB_POOL};

#[derive(sqlx::FromRow)]
pub struct BigMoji {
    pub name: String,
    pub text: String,
    pub inserted_at: NaiveDateTime,
}

#[instrument]
pub async fn add(interaction: &Interaction) -> anyhow::Result<String> {
    let cmd = get_cmd(interaction)?;

    let mut name = cmd.name.replace(':', "").to_lowercase();
    name.retain(|c| !c.is_whitespace());

    let text = cmd
        .value
        .as_str()
        .ok_or(anyhow!("bigmoji text"))?
        .to_string();

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
    .execute(&*DB_POOL)
    .await
    .with_context(|| "inserting bigmoji")?;

    Ok(format!("BigMoji `:{}:` added", name))
}

pub async fn remove(interaction: &Interaction) -> anyhow::Result<String> {
    let cmd = get_cmd(interaction)?;

    let mut name = cmd.name.replace(':', "").to_lowercase();
    name.retain(|c| !c.is_whitespace());

    sqlx::query!("DELETE FROM bigmoji WHERE name = ?;", name)
        .execute(&*DB_POOL)
        .await
        .with_context(|| "deleting bigmoji")?;

    Ok(format!("Deleted BigMoji `:{}:`", name))
}

pub async fn get(interaction: &Interaction) -> anyhow::Result<String> {
    let cmd = get_cmd(interaction)?;

    let mut name = cmd.name.replace(':', "").to_lowercase();
    name.retain(|c| !c.is_whitespace());

    let moji: Option<BigMoji> =
        sqlx::query_as!(BigMoji, "SELECT * FROM bigmoji WHERE name = ?;", name)
            .fetch_optional(&*DB_POOL)
            .await
            .with_context(|| "getting bigmoji")?;

    if let Some(moji) = moji {
        Ok(moji.text)
    } else {
        Ok(format!("BigMoji `:{}:` does not exist", name))
    }
}
