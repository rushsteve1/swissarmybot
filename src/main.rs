use std::env;
use std::sync::Arc;

use log::{debug, error, info, warn};
use once_cell::sync::OnceCell;
use serenity::all::ApplicationId;
use serenity::http::Http;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use sqlx::migrate::MigrateDatabase;

mod commands;
mod jobs;
mod web;

use commands::Handler;
use jobs::setup_jobs;

// Get version and git info from environment variables
pub const VERSION: &str = std::env!("CARGO_PKG_VERSION");
pub const GIT_VERSION: Option<&'static str> = std::option_env!("GIT_VERSION");

lazy_static::lazy_static! {
    // Get configuration from environment variables
    // These make working with SAB in a docker container much easier
    pub static ref TOKEN: String  = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN env variable");
    pub static ref PORT: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT is not a number");
    pub static ref APP_ID: ApplicationId = env::var("APPLICATION_ID")
        .expect("Missing APPLICATION_ID env variable")
        .parse()
        .expect("APPLICATION_ID is not a number");
    pub static ref DB_PATH: String = env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::temp_dir()
            .join("swissarmy.sqlite")
            .into_os_string()
            .into_string()
            .unwrap()
    });

    pub static ref DOMAIN: String = env::var("WEB_DOMAIN").unwrap_or_else(|_| "0.0.0.0".to_string());
    pub static ref PREFIX: String = env::var("ROUTE_PREFIX").unwrap_or_default();

    pub static ref QOTD_CHANNELS: Vec<ChannelId> =
        env::var("QOTD_CHANNELS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().parse::<u64>().unwrap().into())
            .collect();
    pub static ref STONKS_CHANNELS: Vec<ChannelId> =
        env::var("STONKS_CHANNELS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().parse::<u64>().unwrap().into())
            .collect();

    // Build and connect to the database
    pub static ref DB_POOL: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_lazy(&DB_PATH)
        .expect("Error connecting to database");
}

pub static HTTP: OnceCell<Arc<Http>> = OnceCell::new();

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting up SwissArmyBot {}...", VERSION);

    // Check the database path properly, creating the database if needed
    let path_e = std::fs::canonicalize(&*DB_PATH);
    if let Err(ref e) = path_e {
        match e.kind() {
            std::io::ErrorKind::NotFound => {
                sqlx::Sqlite::create_database(&DB_PATH)
                    .await
                    .expect("Error creating database");
            }
            _ => {
                path_e.expect("DATABASE_PATH is not a valid path");
            }
        }
    }

    debug!("Configuration loaded from env variables");
    info!("Using database file {}", *DB_PATH);

    // Apply migrations
    sqlx::migrate!("./migrations")
        .run(&*DB_POOL)
        .await
        .expect("Failed to migrate database");

    info!("Database migration completed");

    // Build the Serenity client
    let mut client = Client::builder(TOKEN.clone(), GatewayIntents::default())
        .event_handler(Handler)
        .application_id(*APP_ID)
        .await
        .expect("Error creating client");

    HTTP.set(client.http.clone()).unwrap();

    let client_fut = client.start();

    // Build the Gotham server
    let addr = format!("0.0.0.0:{}", *PORT);
    info!("Binding to address `{}`", addr);
    let gotham_fut = gotham::plain::init_server(addr, web::router());

    let mut scheduler = setup_jobs();
    let job_fut = tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(std::time::Duration::from_millis(10_000)).await;
        }
    });

    // We're running both Serenity and Gotham in Tokio workers, and neither of
    // them should ever exit, so we wait for them and print an error if they do.
    debug!("Starting event loop...");
    loop {
        tokio::select!(
            e = client_fut => {
                error!("Serenity exited with {:?}", e.unwrap_err());
                break;
            }
            e = gotham_fut => {
                error!("Gotham exited with {:?}", e.unwrap_err());
                break;
            }
            e = job_fut => {
                error!("Clokwerk exited with {:?}", e.unwrap_err());
                break;
            }
        )
    }

    // If it gets to this point then it has exited abnormally
    error!("SwissArmyBot has exited, whoops!");
    std::process::exit(1);
}
