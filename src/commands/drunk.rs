use anyhow::Context;
use chrono::NaiveDateTime;
use serenity::all::{CommandDataOptionValue, Interaction, Mentionable, UserId};
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
    pub water: i64,
    pub updated_at: NaiveDateTime,
    pub score: i64,
    pub last_drink: Option<String>,
    pub last_spill: Option<NaiveDateTime>,
}

impl Drunk {
    pub fn last_spill_str(&self) -> String {
        self.last_spill.map_or("N/A".to_string(), |o| o.to_string())
    }
}

pub async fn update(db: sqlx::SqlitePool, interaction: &Interaction) -> anyhow::Result<String> {
    let inter = get_inter(interaction)?;
    let cmd = get_cmd(interaction)?;

    let author = inter
        .member
        .as_ref()
        .ok_or(anyhow::anyhow!("interaction had no author"))?;
    let author_id = author.user.id;
    let author_name = author.user.name.to_string();

    let drink_type = cmd.name.as_str();
    let CommandDataOptionValue::SubCommand(subcmds) = cmd.value.clone() else {
        error!("command value was not a subcommand");
        return Err(anyhow::anyhow!("command value was not a subcommand"));
    };
    let drink_name = subcmds.first().and_then(|d| d.value.as_str());

    update_handler(db, author_id, author_name.as_str(), drink_type, drink_name).await
}

#[instrument]
async fn update_handler(
    db: sqlx::SqlitePool,
    author_id: UserId,
    author_name: &str,
    drink_type: &str,
    drink_name: Option<&str>,
) -> anyhow::Result<String> {
    let author_id_s = author_id.to_string();

    sqlx::query!(
        "INSERT INTO drunk (user_id, user_name) VALUES (?, ?) ON CONFLICT (user_id) DO NOTHING;",
        author_id_s,
        author_name
    )
    .execute(&db)
    .await
    .with_context(|| "inserting drunk")?;

    // Repetitive, but that's the price of compile-time SQL validation
    match drink_type {
            "beer" => sqlx::query!(
                "UPDATE drunk SET beer = beer + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "wine" => sqlx::query!(
                "UPDATE drunk SET wine = wine + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "shot" => sqlx::query!(
                "UPDATE drunk SET shots = shots + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "cocktail" => sqlx::query!(
                "UPDATE drunk SET cocktails = cocktails + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "derby" => sqlx::query!(
                "UPDATE drunk SET derby = derby + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "water" => sqlx::query!(
                "UPDATE drunk SET water = water + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            _ => {
                error!("unknown drink type");
                return Err(anyhow::anyhow!("unknown drink type"));
            },
        }
        .execute(&db)
        .await
        .with_context(|| "updating drunk")?;

    let type_str = if let Some(name) = drink_name {
        format!("{}: {}", drink_type, name)
    } else {
        drink_type.to_string()
    };

    sqlx::query!(
        "UPDATE drunk SET last_drink = ? WHERE user_id = ?;",
        type_str,
        author_id_s
    )
    .execute(&db)
    .await
    .with_context(|| "updating last_drink")?;

    Ok(format!("{} had a {}", author_id.mention(), type_str))
}
