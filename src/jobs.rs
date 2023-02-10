use crate::web::{post_random_to_channel, post_stonks_to_channel};
use chrono::FixedOffset;
use clokwerk::Interval;
use clokwerk::{AsyncScheduler, Job, TimeUnits};

use crate::{QOTD_CHANNELS, STONKS_CHANNELS};

pub fn setup_jobs() -> AsyncScheduler {
    let mut scheduler = AsyncScheduler::with_tz(FixedOffset::east_opt(5 * 3600).unwrap());

    // Quote of the Day schedule
    scheduler.every(1.day()).at("5:00 am").run(|| async {
        warn!("Running QOTD Job");
        for chan in QOTD_CHANNELS.iter() {
            post_random_to_channel(*chan, "**Quote of the Day**".to_string())
                .await
                .expect("Could not post random quote (in job)");
        }
    });

    // Daily Stonks schedule
    scheduler
        .every(Interval::Weekday)
        .at("5:00 pm")
        .run(|| async {
            warn!("Running Stonks Job");
            for chan in STONKS_CHANNELS.iter() {
                post_stonks_to_channel(*chan)
                    .await
                    .expect("Could not post stonks (in job)");
            }
        });

    return scheduler;
}
