use std::env;
use std::sync::Arc;

use clokwerk::Interval;
use clokwerk::{AsyncScheduler, Job};
use poise::serenity_prelude::{ChannelId, Http};
use tracing::{error, instrument, trace_span, warn};

use crate::shared::helpers::post_stonks_to_channel;

#[instrument]
pub fn setup_jobs(db: sqlx::PgPool, http: Arc<Http>) -> AsyncScheduler<chrono_tz::Tz> {
	let mut scheduler = AsyncScheduler::with_tz(chrono_tz::America::New_York);

	// Calculate these here just the once
	let stonks_channels: Vec<ChannelId> = env::var("STONKS_CHANNELS")
		.unwrap_or_default()
		.split(',')
		.filter_map(|s| s.trim().parse::<u64>().ok().map(std::convert::Into::into))
		.collect();

	// Daily Stonks schedule
	scheduler
		.every(Interval::Weekday)
		.at("5:00 pm")
		.run(move || {
			let stonks_channels = stonks_channels.clone();
			let http = http.clone();

			async move {
				let span = trace_span!("Stonks job");
				let _enter = span.enter();

				for chan in stonks_channels {
					let Ok(_) = post_stonks_to_channel(http.clone(), chan).await else {
						error!("could not post stonks");
						break;
					};
				}
			}
		});

	scheduler
}
