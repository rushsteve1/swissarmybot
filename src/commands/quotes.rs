use serenity::model::prelude::*;
use serenity::model::application::interaction::Interaction;
use sqlx::Row;

use super::{get_cmd, AsInner};
use crate::models::Quote;
use crate::{DB_POOL, DOMAIN, PREFIX};

pub async fn add(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    let user = cmd
        .options
        .first()
        .unwrap()
        .resolved
        .as_ref()
        .unwrap()
        .as_user()
        .unwrap();

    let text = cmd
        .options
        .last()
        .unwrap()
        .resolved
        .as_ref()
        .unwrap()
        .as_string()
        .unwrap();

    if let Interaction::ApplicationCommand(inter) = interaction {
        let author = inter.member.as_ref().unwrap();

        sqlx::query("INSERT INTO quotes (text, user_id, user_name, author_id, author_name) VALUES (?, ?, ?, ?, ?);")
                        .bind(&text)
                        .bind(user.id.0.to_string())
                        .bind(&user.name)
                        .bind(author.user.id.to_string())
                        .bind(author.user.name.to_string())
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
    let id = cmd
        .options
        .first()
        .unwrap()
        .resolved
        .as_ref()
        .unwrap()
        .as_int()
        .unwrap();

    let row = sqlx::query("DELETE FROM quotes WHERE id = ? RETURNING user_id;")
        .bind(id)
        .fetch_optional(&*DB_POOL)
        .await
        .expect("Error deleting quote");

    if let Some(row) = row {
        let user_id: i64 = row.get("user_id");
        let user_id = serenity::model::id::UserId(user_id as u64);

        format!("Quote {} removed by {}", id, user_id.mention())
    } else {
        format!("Quote {} does not exist", id)
    }
}

pub async fn get(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);
    let id = cmd
        .options
        .first()
        .unwrap()
        .resolved
        .as_ref()
        .unwrap()
        .as_int()
        .unwrap();

    let quote: Option<Quote> = sqlx::query_as("SELECT * FROM quotes WHERE id = ?;")
        .bind(id)
        .fetch_optional(&*DB_POOL)
        .await
        .expect("Error getting quote");

    // TODO embed reponse
    if let Some(quote) = quote {
        format!(
            "Quote {} by {}\n>>> {}",
            id,
            UserId(quote.user_id as u64).mention(),
            quote.text
        )
    } else {
        format!("Quote {} does not exist", id)
    }
}

pub async fn list(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    if let Some(user) = cmd.options.first() {
        let user = user.resolved.as_ref().unwrap().as_user().unwrap();
        format!("http://{}{}/quotes?user={}", *DOMAIN, *PREFIX, user.id)
    } else {
        format!("http://{}{}/quotes", *DOMAIN, *PREFIX)
    }
}
