use chrono::NaiveDateTime;
use serenity::all::{Interaction, Mentionable};
use tracing::instrument;

use super::get_cmd;
use crate::DB_POOL;

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
}

#[instrument]
pub async fn update(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    if let Interaction::Command(inter) = interaction {
        let author = inter.member.as_ref().unwrap();

        let author_id = author.user.id.to_string();
        let author_name = author.user.name.to_string();

        sqlx::query!(
            "INSERT INTO drunk (user_id, user_name) VALUES (?, ?) ON CONFLICT (user_id) DO NOTHING;",
            author_id,
            author_name
        )
        .execute(&*DB_POOL)
        .await
        .expect("Error inserting drunk");

        // Repetitive, but that's the price of compile-time SQL validation
        match cmd.name.as_str() {
            "beer" => sqlx::query!(
                "UPDATE drunk SET beer = beer + 1 WHERE user_id = ?;",
                author_id
            ),
            "wine" => sqlx::query!(
                "UPDATE drunk SET wine = wine + 1 WHERE user_id = ?;",
                author_id
            ),
            "shot" => sqlx::query!(
                "UPDATE drunk SET shots = shots + 1 WHERE user_id = ?;",
                author_id
            ),
            "cocktail" => sqlx::query!(
                "UPDATE drunk SET cocktails = cocktails + 1 WHERE user_id = ?;",
                author_id
            ),
            "derby" => sqlx::query!(
                "UPDATE drunk SET derby = derby + 1 WHERE user_id = ?;",
                author_id
            ),
            _ => unreachable!(),
        }
        .execute(&*DB_POOL)
        .await
        .expect("Error updating drunk");

        format!("{} had a {}", author.mention(), cmd.name.as_str())
    } else {
        unreachable!()
    }
}
