use scraper::{Html, Selector};
use serenity::{
    all::{ChannelId, Mentionable, Message},
    builder::CreateMessage,
};
use tracing::instrument;

use crate::{
    commands::{BigMoji, Drunk, Quote},
    DB_POOL, HTTP,
};

const GOOD_STONKS: &str = "📈";
const BAD_STONKS: &str = "📉";
const STONKS_URL: &str = "https://finance.yahoo.com";
const STONKS_SEL: &str = "#marketsummary-itm-2 > h3:nth-child(1) > div:nth-child(4) > fin-streamer:nth-child(1) > span:nth-child(1)";

#[instrument]
pub async fn get_bigmoji() -> Vec<BigMoji> {
    sqlx::query_as!(BigMoji, "SELECT * FROM bigmoji;")
        .fetch_all(&*DB_POOL)
        .await
        .expect("Error getting bigmoji")
}

#[instrument]
pub async fn get_quotes(
    from_date: String,
    to_date: String,
    user_id: i64,
) -> (Vec<Quote>, i64, String, String) {
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
        .expect("Error getting quotes")
    } else {
        sqlx::query_as!(
            Quote,
            "SELECT * FROM quotes WHERE inserted_at BETWEEN ? AND ?;",
            from_date,
            to_date
        )
        .fetch_all(&*DB_POOL)
        .await
        .expect("Error getting quotes")
    };

    (quotes, user_id, from_date, to_date)
}

#[instrument]
pub async fn get_drunks() -> Vec<Drunk> {
    sqlx::query_as!(Drunk, "SELECT * FROM drunk;")
        .fetch_all(&*DB_POOL)
        .await
        .expect("Error getting drunks")
}

#[instrument]
pub async fn get_random_quote() -> Quote {
    sqlx::query_as!(Quote, "SELECT * FROM quotes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(&*DB_POOL)
        .await
        .expect("Error getting quote")
}

#[instrument]
pub async fn post_random_to_channel(
    chan: ChannelId,
    body: String,
) -> Result<Message, serenity::Error> {
    let quote = get_random_quote().await;
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

    chan.send_message(HTTP.get().unwrap(), CreateMessage::new().content(txt))
        .await
}

#[instrument]
pub async fn post_stonks_to_channel(chan: ChannelId) -> Result<Message, serenity::Error> {
    let txt = {
        let body = reqwest::get(STONKS_URL)
            .await
            .expect("Failed to get Yahoo Finance")
            .text()
            .await
            .expect("Could not get Stonks body");
        let document = Html::parse_document(&body);
        let selector = Selector::parse(STONKS_SEL).expect("Failed to parse selector");
        let el = document.select(&selector).next().unwrap();
        let c = el.text().next().unwrap().chars().next().unwrap();
        match c {
            '+' => GOOD_STONKS,
            '-' => BAD_STONKS,
            _ => unreachable!(),
        }
    };

    chan.send_message(HTTP.get().unwrap(), CreateMessage::new().content(txt))
        .await
}
