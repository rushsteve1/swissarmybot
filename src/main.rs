#![forbid(unsafe_code)]
#![forbid(future_incompatible)]
#![forbid(clippy::indexing_slicing)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::invalid_regex)]

use std::{env, future::IntoFuture, time::Duration};

use anyhow::{bail, Context};
use serenity::all::ApplicationId;
use serenity::prelude::*;
use sqlx::{migrate::MigrateDatabase, SqlitePool};

use opentelemetry::KeyValue;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
	runtime,
	trace::{self, Sampler},
	Resource,
};
use tracing::{debug, info, instrument, warn};
use tracing_subscriber::layer::SubscriberExt;

mod commands;
mod jobs;
mod shared;
mod web;

use commands::Handler;
use jobs::setup_jobs;
use web::router;

// Get version and git info from environment variables during compile
pub const VERSION: &str = std::env!("CARGO_PKG_VERSION");
pub const GIT_VERSION: Option<&'static str> = std::option_env!("GIT_VERSION");

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
	// Before we do anything set Tracing to use stdout
	tracing_subscriber::fmt::init();

	let cfg = setup_config()?;

	setup_tracing(cfg.clone())?;

	info!("Starting up SwissArmyBot {}...", VERSION);

	let db_pool = setup_db().await?;

	// Build the Serenity client
	let mut client = Client::builder(cfg.token.clone(), GatewayIntents::default())
		.type_map_insert::<DB>(db_pool.clone())
		.type_map_insert::<Config>(cfg.clone())
		.event_handler(Handler)
		.application_id(cfg.app_id)
		.await?;

	// Build the Axum server
	info!("Binding to address `{}`", cfg.addr());
	let listener = tokio::net::TcpListener::bind(cfg.addr()).await?;
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

#[derive(Clone, Debug, Default)]
pub struct Config {
	pub token: String,
	pub app_id: ApplicationId,
	pub domain: String,
	pub port: u16,
	pub otel_endpoint: String,
	pub otel_api_key: String,
}

impl TypeMapKey for Config {
	type Value = Self;
}

impl Config {
	pub fn addr(&self) -> String {
		format!("{}:{}", self.domain, self.port)
	}
}

// Get configuration from environment variables
// These make working with SAB in a docker container much easier
fn setup_config() -> anyhow::Result<Config> {
	let Ok(token) = env::var("DISCORD_TOKEN") else {
		bail!("Missing DISCORD_TOKEN env variable");
	};
	info!("Discord token set");

	let Ok(port) = env::var("PORT").map_or(Ok(8080), |p| p.parse::<u16>()) else {
		bail!("PORT is not a number");
	};
	info!("Using port: {}", port);

	let Ok(app_id) = env::var("APPLICATION_ID") else {
		bail!("APPLICATION_ID is not set");
	};
	let Ok(app_id) = app_id.parse::<ApplicationId>() else {
		bail!("APPLICATION_ID is invalid");
	};
	info!("Application ID set");

	let domain = env::var("WEB_DOMAIN").unwrap_or_else(|_| "0.0.0.0".to_string());
	info!("Using domain: {}", domain);

	let otel_endpoint = env::var("OTEL_ENDPOINT").unwrap_or_default();
	info!("Using OTEL endpoint: {}", otel_endpoint);
	let otel_api_key = env::var("OTEL_API_KEY").unwrap_or_default();

	Ok(Config {
		token,
		app_id,
		domain,
		port,
		otel_endpoint,
		otel_api_key,
	})
}

fn setup_tracing(cfg: Config) -> anyhow::Result<()> {
	if cfg.otel_endpoint.is_empty() {
		return Ok(());
	}

	let mut metamap = tonic::metadata::MetadataMap::with_capacity(2);
	metamap.insert("x-host", cfg.domain.parse()?);
	metamap.insert("api-key", cfg.otel_api_key.parse()?);

	let exporter = opentelemetry_otlp::new_exporter()
		.tonic()
		.with_protocol(Protocol::Grpc)
		.with_endpoint(cfg.otel_endpoint)
		.with_timeout(Duration::from_secs(3))
		.with_metadata(metamap);

	let tracer = opentelemetry_otlp::new_pipeline()
		.tracing()
		.with_exporter(exporter)
		.with_trace_config(
			trace::config()
				.with_sampler(Sampler::AlwaysOn)
				.with_max_attributes_per_span(16)
				.with_max_events_per_span(16)
				.with_resource(Resource::new(vec![KeyValue::new(
					"service.name",
					"swiss_army_bot",
				)])),
		)
		.install_batch(runtime::Tokio)?;

	let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
	let subscriber = tracing_subscriber::Registry::default().with(telemetry);

	tracing::subscriber::set_global_default(subscriber).with_context(|| "tracing subscriber")
}

pub struct DB;

impl TypeMapKey for DB {
	type Value = sqlx::SqlitePool;
}

async fn setup_db() -> anyhow::Result<SqlitePool> {
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

	Ok(db_pool)
}
