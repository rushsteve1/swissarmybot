use juniper::graphql_object;
use sqlx::SqlitePool;

use crate::shared::{drunks, quotes, Drunk, Quote};

pub struct Context {
	db: SqlitePool,
}

impl juniper::Context for Context {}

pub struct Query;

#[graphql_object(Context = Context)]
impl Query {
	fn apiVersion() -> &str {
		"1.0"
	}

	async fn quotes(ctx: &Context) -> Vec<Quote> {
		quotes::get_all(&ctx.db).await.unwrap_or_default()
	}

	async fn drunks(ctx: &Context) -> Vec<Drunk> {
		drunks::get_all(&ctx.db).await.unwrap_or_default()
	}
}
