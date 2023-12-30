use anyhow::Context;
use chrono::NaiveDateTime;
use serenity::all::{CommandDataOptionValue, Interaction, Mentionable};
use tracing::{error, instrument};

use crate::helpers::{get_cmd, get_inter};

#[derive(sqlx::FromRow)]
pub struct Drunk {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub beer: i64,
    pub wine: i64,
    pub shots: i64,
    pub cocktails: i64,
    pub derby: i64,
    pub updated_at: NaiveDateTime,
    pub score: i64,
    pub last_drink: Option<String>,
    pub last_spill: Option<NaiveDateTime>,
}

#[instrument]
pub async fn update(db: sqlx::SqlitePool, interaction: &Interaction) -> anyhow::Result<String> {
    let inter = get_inter(interaction)?;
    let cmd = get_cmd(interaction)?;

    let author = inter
        .member
        .as_ref()
        .ok_or(anyhow::anyhow!("interaction had no author"))?;
    let author_id = author.user.id.to_string();
    let author_name = author.user.name.to_string();

    let CommandDataOptionValue::SubCommand(subcmds) = cmd.value.clone() else {
        error!("command value was not a subcommand");
        return Err(anyhow::anyhow!("command value was not a subcommand"));
    };
    let drink_name = subcmds.get(0).and_then(|d| d.value.as_str());

    sqlx::query!(
        "INSERT INTO drunk (user_id, user_name) VALUES (?, ?) ON CONFLICT (user_id) DO NOTHING;",
        author_id,
        author_name
    )
    .execute(&db)
    .await
    .with_context(|| "inserting drunk")?;

    // Repetitive, but that's the price of compile-time SQL validation
    match cmd.name.as_str() {
            "beer" => sqlx::query!(
                "UPDATE drunk SET beer = beer + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id
            ),
            "wine" => sqlx::query!(
                "UPDATE drunk SET wine = wine + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id
            ),
            "shot" => sqlx::query!(
                "UPDATE drunk SET shots = shots + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id
            ),
            "cocktail" => sqlx::query!(
                "UPDATE drunk SET cocktails = cocktails + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id
            ),
            "derby" => sqlx::query!(
                "UPDATE drunk SET derby = derby + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id
            ),
            _ => {
                error!("unknown drink type");
                return Err(anyhow::anyhow!("unknown drink type"));
            },
        }
        .execute(&db)
        .await
        .with_context(|| "updating drunk")?;

    if let Some(name) = drink_name {
        sqlx::query!(
            "UPDATE drunk SET last_drink = ? WHERE user_id = ?;",
            name,
            author_id
        )
        .execute(&db)
        .await
        .with_context(|| "updating last_drink")?;

        Ok(format!(
            "{} had a {} [`{}`]",
            author.mention(),
            cmd.name.as_str(),
            name
        ))
    } else {
        sqlx::query!(
            "UPDATE drunk SET last_drink = NULL WHERE user_id = ?;",
            author_id
        )
        .execute(&db)
        .await
        .with_context(|| "clearing last_drink")?;

        Ok(format!("{} had a {}", author.mention(), cmd.name.as_str()))
    }
}
