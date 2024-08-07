use std::{env, future::IntoFuture};

use anyhow::{bail, Context};
use poise::serenity_prelude as serenity;
use sqlx::{migrate::MigrateDatabase, SqlitePool};

use tracing::{debug, info, instrument, warn};

mod commands;
mod jobs;
mod shared;
mod web;

use jobs::setup_jobs;
use web::router;

use crate::commands::events::handler;

// Get version and git info from environment variables during compile
pub const VERSION: &str = std::env!("CARGO_PKG_VERSION");
pub const GIT_VERSION: Option<&'static str> = std::option_env!("GIT_VERSION");

const OWNER: serenity::UserId = serenity::UserId::new(114_901_572_084_826_119);

#[derive(Debug, Clone)]
struct Data {
	db: SqlitePool,
	cfg: Config,
}
type Ctx<'a> = poise::Context<'a, Data, anyhow::Error>;

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();

	let cfg = setup_config().with_context(|| "config setup")?;

	info!("Starting up SwissArmyBot {}...", VERSION);

	let db_pool = setup_db().await.with_context(|| "database setup")?;

	// Build the Axum server
	let addr = format!("0.0.0.0:{}", cfg.port);
	info!("Binding to address `{}`", addr);
	let listener = tokio::net::TcpListener::bind(addr)
		.await
		.with_context(|| "TCP listener setup")?;
	let axum_fut = axum::serve(listener, router(db_pool.clone())).into_future();

	if cfg.only_webserver {
		info!("Webserver only mode enabled...");
		return axum_fut.await.with_context(|| "axum server");
	}

	// Makes the borrow checker happy
	let fdb = db_pool.clone();
	let fcfg = cfg.clone();

	let mut owners = std::collections::HashSet::new();
	owners.insert(OWNER);

	// Build the Poise framework
	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			owners,
			event_handler: |ctx, event, framework, data| {
				Box::pin(handler(ctx, event, framework, data))
			},
			commands: vec![
				commands::register(),
				commands::drunks::drunk(),
				commands::drunks::spill(),
				commands::quotes::top(),
				commands::quotes::context_menu(),
				commands::uiua::uiua(),
				commands::uiua::context_menu(),
			],
			..Default::default()
		})
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(Data { db: fdb, cfg: fcfg })
			})
		})
		.build();

	// Build the Serenity client
	let mut client = serenity::ClientBuilder::new(
		cfg.token.clone(),
		serenity::GatewayIntents::non_privileged(),
	)
	.framework(framework)
	.application_id(cfg.app_id.unwrap_or_default())
	.await
	.with_context(|| "serenity client setup")?;

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

	// Fixes a clippy lint, have to put it around a block so it applies to the macro
	#[allow(clippy::redundant_pub_crate)]
	{
		tokio::select!(
			e = serenity_fut => e.with_context(|| "Serenity exited!")?,
			e = axum_fut => e.with_context(|| "Axum exited!")?,
			e = job_fut => e.with_context(|| "Jobs exited!")?,
		);
	}

	// Shouldn't be possible, but just in case
	anyhow::bail!("SwissArmyBot exited!")
}

#[derive(Clone, Debug, Default)]
pub struct Config {
	pub token: String,
	pub app_id: Option<serenity::ApplicationId>,
	pub domain: String,
	pub port: u16,
	pub otel_endpoint: String,
	pub otel_api_key: String,
	pub only_webserver: bool,
}

// Get configuration from environment variables
// These make working with SAB in a docker container much easier
fn setup_config() -> anyhow::Result<Config> {
	dotenvy::dotenv()?;

	let Ok(token) = env::var("DISCORD_TOKEN") else {
		bail!("Missing DISCORD_TOKEN env variable");
	};
	info!("Discord token set");

	let Ok(port) = env::var("PORT").map_or(Ok(8080), |p| p.parse::<u16>()) else {
		bail!("PORT is not a number");
	};
	info!("Using port: {}", port);

	let app_id = env::var("APPLICATION_ID")
		.with_context(|| "APPLICATION_ID env variable")
		.and_then(|id| {
			id.parse::<serenity::ApplicationId>()
				.with_context(|| "APPLICATION_ID parse")
		})
		.ok();
	info!("Application ID set");

	let domain = env::var("WEB_DOMAIN").unwrap_or_else(|_| "0.0.0.0".to_string());
	info!("Using domain: {}", domain);

	let otel_endpoint = env::var("OTEL_ENDPOINT").unwrap_or_default();
	info!("Using OTEL endpoint: {}", otel_endpoint);
	let otel_api_key = env::var("OTEL_API_KEY").unwrap_or_default();

	let only_webserver = env::var("ONLY_WEBSERVER").is_ok();

	Ok(Config {
		token,
		app_id,
		domain,
		port,
		otel_endpoint,
		otel_api_key,
		only_webserver,
	})
}

async fn setup_db() -> anyhow::Result<SqlitePool> {
	// Build and connect to the database
	let db_path = env::var("DATABASE_URL").unwrap_or_else(|_| "./swissarmy.sqlite".to_string());

	// Check the database path properly, creating the database if needed
	let path_e = std::fs::canonicalize(&db_path);
	if let Err(ref e) = path_e {
		match e.kind() {
			std::io::ErrorKind::NotFound => {
				sqlx::Sqlite::create_database(&db_path)
					.await
					.with_context(|| "database creation")?;
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

	Ok(db_pool)
}
