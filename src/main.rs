use std::env;

use anyhow::{bail, Context};
use poise::serenity_prelude as serenity;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, instrument};

mod commands;
mod quotes;

use crate::commands::events::handler;
use crate::quotes::create_table;

// Get version and git info from environment variables during compile
pub const VERSION: &str = std::env!("CARGO_PKG_VERSION");
pub const GIT_VERSION: Option<&'static str> = std::option_env!("GIT_VERSION");

const OWNER: serenity::UserId = serenity::UserId::new(114_901_572_084_826_119);

#[derive(Debug, Clone)]
struct Data {
	db: PgPool,
	cfg: Config,
}
type Ctx<'a> = poise::Context<'a, Data, anyhow::Error>;

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
	if cfg!(feature = "tracy") {
		#[cfg(feature = "tracy")]
		{
			use tracing_subscriber::layer::SubscriberExt;
			tracing::subscriber::set_global_default(
				tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
			)
			.expect("setup tracy layer");
		}
	} else {
		tracing_subscriber::fmt::init();
	}

	let cfg = setup_config().with_context(|| "config setup")?;

	info!("Starting up SwissArmyBot {}...", VERSION);

	let db_pool = setup_db().await.with_context(|| "database setup")?;

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
				commands::quotes::top::<anyhow::Error>(),
				commands::quotes::context_menu(),
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

	// Build the Serenity client and start it, blocking the main thread
	serenity::ClientBuilder::new(
		cfg.token.clone(),
		serenity::GatewayIntents::non_privileged(),
	)
	.framework(framework)
	.application_id(cfg.app_id.unwrap_or_default())
	.await
	.with_context(|| "serenity client setup")?
	.start()
	.await?;

	unreachable!()
}

#[derive(Clone, Debug, Default)]
pub struct Config {
	pub token: String,
	pub app_id: Option<serenity::ApplicationId>,
	pub otel_endpoint: String,
	pub otel_api_key: String,
}

// Get configuration from environment variables
// These make working with SAB in a docker container much easier
fn setup_config() -> anyhow::Result<Config> {
	dotenvy::dotenv()?;

	let Ok(token) = env::var("DISCORD_TOKEN") else {
		bail!("Missing DISCORD_TOKEN env variable");
	};
	info!("Discord token set");

	let app_id = env::var("APPLICATION_ID")
		.with_context(|| "APPLICATION_ID env variable")
		.and_then(|id| {
			id.parse::<serenity::ApplicationId>()
				.with_context(|| "APPLICATION_ID parse")
		})
		.ok();
	info!("Application ID set");

	let otel_endpoint = env::var("OTEL_ENDPOINT").unwrap_or_default();
	info!("Using OTEL endpoint: {}", otel_endpoint);
	let otel_api_key = env::var("OTEL_API_KEY").unwrap_or_default();

	Ok(Config {
		token,
		app_id,
		otel_endpoint,
		otel_api_key,
	})
}

async fn setup_db() -> anyhow::Result<PgPool> {
	// Build and connect to the database
	let db_url = env::var("DATABASE_URL").with_context(|| "Connecting to database")?;

	let db_pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(&db_url)
		.await
		.with_context(|| "Error connecting to database")?;

	create_table(&db_pool).await?;

	info!("Database migration completed");

	Ok(db_pool)
}
