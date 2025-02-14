use std::sync::Arc;

use anyhow::Context;
use poise::serenity_prelude::{ChannelId, CreateMessage, Http, Message, UserId};
use tracing::instrument;

const GOOD_STONKS: &str = "📈";
const BAD_STONKS: &str = "📉";

// TODO need to find a better API than scraping Yahoo
const STONKS_URL: &str = "https://query2.finance.yahoo.com/v8/finance/chart/NQ%3DF";

#[instrument]
pub async fn post_stonks_to_channel(http: Arc<Http>, chan: ChannelId) -> anyhow::Result<Message> {
	let raw_data: serde_json::Value = reqwest::get(STONKS_URL).await?.json().await?;
	let meta = &raw_data["chart"]["result"][0]["meta"];
	let current = meta["regularMarketPrice"].as_f64().unwrap();
	let previous = meta["previousClose"].as_f64().unwrap();

	let emoji = if current > previous {
		GOOD_STONKS
	} else {
		BAD_STONKS
	};

	return chan
		.send_message(http, CreateMessage::new().content(emoji))
		.await
		.with_context(|| "stonks send");
}

pub fn to_userid(s: impl Into<String>) -> UserId {
	s.into().parse().unwrap_or_default()
}
