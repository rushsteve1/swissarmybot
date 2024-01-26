use std::sync::Arc;

use anyhow::Context;
use juniper::GraphQLScalarValue;
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

	let txt = format!(
		"{}\n#{} by {}, added by {} on <t:{}:f>\n\n>>> {}",
		body,
		quote.id,
		UserId::from(quote.user_id).mention(),
		UserId::from(quote.author_id).mention(),
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

// There are a bunch of odd numeric errors between SQLx, Serenity, and Juniper.
// This type wraps a String and implements a number of conversions to allow it to
// work with all the libraries and be used in composite types.
#[derive(Debug, Clone, GraphQLScalarValue)]
pub struct CleverNum(String);

impl From<String> for CleverNum {
	fn from(value: String) -> Self {
		Self(value)
	}
}

impl From<i64> for CleverNum {
	fn from(value: i64) -> Self {
		Self(value.to_string())
	}
}

impl From<CleverNum> for UserId {
	fn from(value: CleverNum) -> Self {
		value.0.parse().unwrap_or_default()
	}
}

impl std::fmt::Display for CleverNum {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl std::ops::Mul<i64> for CleverNum {
	type Output = i64;

	fn mul(self, rhs: i64) -> Self::Output {
		let lhs: i64 = self.0.parse().unwrap_or_default();
		lhs * rhs
	}
}
