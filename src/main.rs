#![forbid(unsafe_code)]
#![forbid(future_incompatible)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use std::{env, future::IntoFuture};

use anyhow::{bail, Context};
use serenity::all::ApplicationId;
use serenity::prelude::*;
use sqlx::migrate::MigrateDatabase;
use tracing::{debug, info, instrument, warn};

mod commands;
mod helpers;
mod jobs;
mod web;

use commands::Handler;
use jobs::setup_jobs;

use crate::web::router;

// Get version and git info from environment variables during compile
pub const VERSION: &str = std::env!("CARGO_PKG_VERSION");
pub const GIT_VERSION: Option<&'static str> = std::option_env!("GIT_VERSION");

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
    // Setup tracing
    tracing_subscriber::fmt::init();

    info!("Starting up SwissArmyBot {}...", VERSION);

    // Get configuration from environment variables
    // These make working with SAB in a docker container much easier
    let Ok(token) = env::var("DISCORD_TOKEN") else {
        bail!("Missing DISCORD_TOKEN env variable");
    };
    let Ok(port) = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
    else {
        bail!("PORT is not a number");
    };
    let Ok(app_id) = env::var("APPLICATION_ID") else {
        bail!("APPLICATION_ID is not set");
    };
    let Ok(app_id) = app_id.parse::<ApplicationId>() else {
        bail!("APPLICATION_ID is invalid");
    };

    // Build and connect to the database
    let db_path = env::var("DATABASE_URL").unwrap_or_else(|_| "./swissarmy.sqlite".to_string());

    // Check the database path properly, creating the database if needed
    let path_e = std::fs::canonicalize(&db_path);
    if let Err(ref e) = path_e {
        match e.kind() {
            std::io::ErrorKind::NotFound => {
                sqlx::Sqlite::create_database(&db_path).await?;
            }
            _ => {
                path_e?;
            }
        }
    }

    info!("Using database file {}", db_path);

    let Ok(db_pool) = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_lazy(&db_path)
    else {
        bail!("Error connecting to database");
    };

    // Apply migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    info!("Database migration completed");

    // Build the Serenity client
    let mut client = Client::builder(token.clone(), GatewayIntents::default())
        .type_map_insert::<DB>(db_pool.clone())
        .event_handler(Handler)
        .application_id(app_id)
        .await?;

    // Build the Axum server
    let addr = format!("0.0.0.0:{}", port);
    info!("Binding to address `{}`", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let axum_fut = axum::serve(listener, router(db_pool.clone())).into_future();

    // Setup the cron jobs to check every 60 seconds
    let mut scheduler = setup_jobs(db_pool, client.http.clone());
    let job_fut = tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });

    // Start the client but don't await it yet since we need to select all the futures together
    let serenity_fut = client.start();

    // We're running everything in Tokio workers, and none of them should ever exit,
    // so we wait for them and print an error if they do.
    debug!("Starting event loop...");
    tokio::select!(
        e = serenity_fut => e.with_context(|| "Serenity exited!")?,
        e = axum_fut => e.with_context(|| "Axum exited!")?,
        e = job_fut => e.with_context(|| "Jobs exited!")?,
    );

    // Shouldn't be possible, but just in case
    anyhow::bail!("SwissArmyBot exited!")
}

pub struct DB;

impl TypeMapKey for DB {
    type Value = sqlx::SqlitePool;
}
