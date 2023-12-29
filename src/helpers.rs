use anyhow::Context;
use scraper::{Html, Selector};
use serenity::{
    all::{ChannelId, CommandDataOption, CommandInteraction, Interaction, Mentionable, Message},
    builder::CreateMessage,
};
use tracing::{instrument, warn};

use crate::{
    commands::{BigMoji, Drunk, Quote},
    DB_POOL, HTTP,
};

const GOOD_STONKS: &str = "📈";
const BAD_STONKS: &str = "📉";
const STONKS_URL: &str = "https://finance.yahoo.com";
const STONKS_SEL: &str = "#marketsummary-itm-2 > h3:nth-child(1) > div:nth-child(4) > fin-streamer:nth-child(1) > span:nth-child(1)";

#[instrument]
pub async fn get_bigmoji(name: String) -> anyhow::Result<Option<BigMoji>> {
    sqlx::query_as!(BigMoji, "SELECT * FROM bigmoji WHERE name = ?;", name)
        .fetch_optional(&*DB_POOL)
        .await
        .with_context(|| "getting BigMoji in message")
}

#[instrument]
pub async fn get_all_bigmoji() -> anyhow::Result<Vec<BigMoji>> {
    sqlx::query_as!(BigMoji, "SELECT * FROM bigmoji;")
        .fetch_all(&*DB_POOL)
        .await
        .with_context(|| "get bigmoji")
}

#[instrument]
pub async fn get_quotes(
    from_date: String,
    to_date: String,
    user_id: i64,
) -> anyhow::Result<(Vec<Quote>, i64, String, String)> {
    let quotes = if user_id > 0 {
        sqlx::query_as!(
            Quote,
            "SELECT * FROM quotes WHERE user_id = ? AND inserted_at BETWEEN ? AND ?;",
            user_id,
            from_date,
            to_date
        )
        .fetch_all(&*DB_POOL)
        .await
    } else {
        sqlx::query_as!(
            Quote,
            "SELECT * FROM quotes WHERE inserted_at BETWEEN ? AND ?;",
            from_date,
            to_date
        )
        .fetch_all(&*DB_POOL)
        .await
    }
    .with_context(|| "getting quotes")?;

    Ok((quotes, user_id, from_date, to_date))
}

#[instrument]
pub async fn get_drunks() -> anyhow::Result<Vec<Drunk>> {
    sqlx::query_as!(
        Drunk,
        "SELECT * FROM drunk ORDER BY (beer + wine + shots + cocktails + derby) DESC;"
    )
    .fetch_all(&*DB_POOL)
    .await
    .with_context(|| "getting drunks")
}

#[instrument]
pub async fn get_random_quote() -> anyhow::Result<Quote> {
    sqlx::query_as!(Quote, "SELECT * FROM quotes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(&*DB_POOL)
        .await
        .with_context(|| "getting quote")
}

#[instrument]
pub async fn post_random_to_channel(chan: ChannelId, body: String) -> anyhow::Result<Message> {
    let quote = get_random_quote().await?;
    let user_id = serenity::model::id::UserId::new(quote.user_id as u64);
    let author_id = serenity::model::id::UserId::new(quote.author_id as u64);

    let txt = format!(
        "{}\n#{} by {}, added by {} on <t:{}:f>\n\n>>> {}",
        body,
        quote.id,
        user_id.mention(),
        author_id.mention(),
        quote.inserted_at.timestamp(),
        quote.text
    );

    let http = HTTP.get().ok_or(anyhow::anyhow!("could not get HTTP"))?;
    chan.send_message(http, CreateMessage::new().content(txt))
        .await
        .with_context(|| "sending random quote")
}

#[instrument]
pub async fn post_stonks_to_channel(chan: ChannelId) -> anyhow::Result<Message> {
    let txt = {
        let body = reqwest::get(STONKS_URL).await?.text().await?;
        let document = Html::parse_document(&body);
        let Ok(selector) = Selector::parse(STONKS_SEL) else {
            anyhow::bail!("selector parsing failed");
        };

        // This is clunky but effective
        (|| {
            let el = document.select(&selector).next()?;
            let c = el.text().next()?.chars().next()?;
            match c {
                '+' => Some(GOOD_STONKS),
                '-' => Some(BAD_STONKS),
                _ => None,
            }
        })()
        .ok_or(anyhow::anyhow!("failed to find element"))?
    };

    let http = HTTP.get().ok_or(anyhow::anyhow!("could not get HTTP"))?;
    chan.send_message(http, CreateMessage::new().content(txt))
        .await
        .with_context(|| "sending stonks message")
}

#[instrument]
pub fn get_inter(interaction: &Interaction) -> anyhow::Result<&CommandInteraction> {
    if let Interaction::Command(inter) = interaction {
        Ok(inter)
    } else {
        warn!("interaction was not command");
        Err(anyhow::anyhow!("interaction was not command"))
    }
}

#[instrument]
pub fn get_cmd(interaction: &Interaction) -> anyhow::Result<&CommandDataOption> {
    get_inter(interaction)?
        .data
        .options
        .first()
        .ok_or(anyhow::anyhow!("interaction did not have command"))
}
