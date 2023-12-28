use clokwerk::Interval;
use clokwerk::{AsyncScheduler, Job, TimeUnits};
use tracing::{instrument, trace_span, warn};

use crate::helpers::{post_random_to_channel, post_stonks_to_channel};
use crate::{QOTD_CHANNELS, STONKS_CHANNELS};

#[instrument]
pub fn setup_jobs() -> AsyncScheduler<chrono_tz::Tz> {
    let mut scheduler = AsyncScheduler::with_tz(chrono_tz::America::New_York);

    // Quote of the Day schedule
    scheduler.every(1.day()).at("5:00 am").run(|| async {
        let span = trace_span!("QOTD job");
        let _enter = span.enter();
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
            let span = trace_span!("Stonks job");
            let _enter = span.enter();
            for chan in STONKS_CHANNELS.iter() {
                post_stonks_to_channel(*chan)
                    .await
                    .expect("Could not post stonks (in job)");
            }
        });

    scheduler
}
