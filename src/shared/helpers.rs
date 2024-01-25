use std::sync::Arc;

use anyhow::Context;
use poise::serenity_prelude::{ChannelId, CreateMessage, Http, Mentionable, Message, UserId};
use scraper::{Html, Selector};
use sqlx::SqlitePool;
use tracing::instrument;

use super::quotes;

const GOOD_STONKS: &str = "📈";
const BAD_STONKS: &str = "📉";
const STONKS_URL: &str = "https://finance.yahoo.com";
const STONKS_SEL: &str = "#marketsummary-itm-2 > h3:nth-child(1) > div:nth-child(4) > fin-streamer:nth-child(1) > span:nth-child(1)";

#[instrument]
pub async fn post_random_to_channel(
	db: &SqlitePool,
	http: Arc<Http>,
	chan: ChannelId,
	body: String,
) -> anyhow::Result<Message> {
	let quote = quotes::get_random(db).await?;
	let user_id = quote.user_id;
	let author_id = quote.author_id;

	let txt = format!(
		"{}\n#{} by {}, added by {} on <t:{}:f>\n\n>>> {}",
		body,
		quote.id,
		user_id.parse::<UserId>()?.mention(),
		author_id.parse::<UserId>()?.mention(),
		quote.inserted_at.timestamp(),
		quote.text
	);

	chan.send_message(http, CreateMessage::new().content(txt))
		.await
		.with_context(|| "sending random quote")
}

#[instrument]
pub async fn post_stonks_to_channel(http: Arc<Http>, chan: ChannelId) -> anyhow::Result<Message> {
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
		.ok_or_else(|| anyhow::anyhow!("failed to find element"))?
	};

	chan.send_message(http, CreateMessage::new().content(txt))
		.await
		.with_context(|| "sending stonks message")
}
