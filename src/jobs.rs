use std::env;
use std::sync::Arc;

use clokwerk::Interval;
use clokwerk::{AsyncScheduler, Job, TimeUnits};
use serenity::all::{ChannelId, Http};
use tracing::{error, instrument, trace_span, warn};

use crate::helpers::{post_random_to_channel, post_stonks_to_channel};

#[instrument]
pub fn setup_jobs(db: sqlx::SqlitePool, http: Arc<Http>) -> AsyncScheduler<chrono_tz::Tz> {
    let mut scheduler = AsyncScheduler::with_tz(chrono_tz::America::New_York);

    // Calculate these here just the once
    let stonks_channels: Vec<ChannelId> = env::var("STONKS_CHANNELS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().parse::<u64>().unwrap().into())
        .collect();
    let qotd_channels: Vec<ChannelId> = env::var("QOTD_CHANNELS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().parse::<u64>().unwrap().into())
        .collect();

    {
        // this needs to be re-used so we have to prevent a move
        let http = http.clone();
        // Quote of the Day schedule
        scheduler.every(1.day()).at("5:00 am").run(move || {
            let qotd_channels = qotd_channels.clone();
            let db = db.clone();
            let http = http.clone();

            async move {
                let span = trace_span!("QOTD job");
                let _enter = span.enter();

                for chan in qotd_channels.into_iter() {
                    let Ok(_) = post_random_to_channel(
                        db.clone(),
                        http.clone(),
                        chan,
                        "**Quote of the Day**".to_string(),
                    )
                    .await
                    else {
                        error!("could not post random quote");
                        break;
                    };
                }
            }
        });
    }

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

                for chan in stonks_channels.into_iter() {
                    let Ok(_) = post_stonks_to_channel(http.clone(), chan).await else {
                        error!("could not post stonks");
                        break;
                    };
                }
            }
        });

    scheduler
}
