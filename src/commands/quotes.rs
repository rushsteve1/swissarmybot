use chrono::NaiveDateTime;
use serenity::model::application::Interaction;
use serenity::model::prelude::*;
use tracing::instrument;

use super::get_cmd;
use crate::{DB_POOL, DOMAIN, HTTP, PREFIX};

#[derive(sqlx::FromRow)]
pub struct Quote {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub author_id: i64,
    pub author_name: String,
    pub text: String,
    pub inserted_at: NaiveDateTime,
}

#[instrument]
pub async fn add(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    let user_id = cmd.value.as_user_id().unwrap();
    let user = user_id.to_user(HTTP.get().unwrap()).await.unwrap();
    let text = cmd.value.as_str().unwrap();

    if let Interaction::Command(inter) = interaction {
        let author = inter.member.as_ref().unwrap();

        let id = user_id.to_string();
        let name = &user.name;
        let author_id = author.user.id.to_string();
        let author_name = author.user.name.to_string();

        sqlx::query!("INSERT INTO quotes (text, user_id, user_name, author_id, author_name) VALUES (?, ?, ?, ?, ?);",
                        text,
                        id, name, author_id, author_name)
                        .execute(&*DB_POOL)
                        .await
                        .expect("Error inserting quote");

        format!("Quote added for {}\n>>> {}", user, text)
    } else {
        unreachable!()
    }
}

pub async fn remove(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);
    let id = cmd.value.as_i64().unwrap();

    let row = sqlx::query_scalar!("DELETE FROM quotes WHERE id = ? RETURNING user_id;", id)
        .fetch_optional(&*DB_POOL)
        .await
        .expect("Error deleting quote");

    if let Some(user_id) = row {
        let user_id = serenity::model::id::UserId::new(user_id as u64);

        format!("Quote {} removed by {}", id, user_id.mention())
    } else {
        format!("Quote {} does not exist", id)
    }
}

#[instrument]
pub async fn get(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);
    let id = cmd.value.as_i64().unwrap();

    let quote: Option<Quote> = sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = ?;", id)
        .fetch_optional(&*DB_POOL)
        .await
        .expect("Error getting quote");

    // TODO embed reponse
    if let Some(quote) = quote {
        format!(
            "Quote {} by {}\n>>> {}",
            id,
            UserId::new(quote.user_id as u64).mention(),
            quote.text
        )
    } else {
        format!("Quote {} does not exist", id)
    }
}

#[instrument]
pub async fn list(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    if let Some(user_id) = cmd.value.as_user_id() {
        format!("http://{}{}/quotes?user={}", *DOMAIN, *PREFIX, user_id)
    } else {
        format!("http://{}{}/quotes", *DOMAIN, *PREFIX)
    }
}
