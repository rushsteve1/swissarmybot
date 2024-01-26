use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Router};
use chrono::Local;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use maud::Markup;
use poise::serenity_prelude::UserId;
use serde::Deserializer;
use serde::{de, Deserialize};
use sqlx::SqlitePool;
use tracing::instrument;

use super::{graphql, templates};

use crate::shared::{drunks, quotes};
use crate::{GIT_VERSION, VERSION};

#[derive(Debug, Deserialize)]
struct QuotesQuery {
	#[serde(default, deserialize_with = "empty_string_as_none")]
	user: Option<u64>,
	from_date: Option<String>,
	to_date: Option<String>,
}

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
	D: Deserializer<'de>,
	T: FromStr,
	T::Err: fmt::Display,
{
	let opt = Option::<String>::deserialize(de)?;
	match opt.as_deref() {
		None | Some("") => Ok(None),
		Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
	}
}

type Schema = RootNode<'static, graphql::Query, EmptyMutation, EmptySubscription>;

pub fn router(db: SqlitePool) -> Router {
	let schema = Schema::new(
		graphql::Query,
		EmptyMutation::new(),
		EmptySubscription::new(),
	);

	Router::new()
		.route("/", get(index))
		.route("/drunks", get(drunks))
		.route("/quotes", get(quotes))
		.fallback(not_found)
		.layer(Extension(Arc::new(schema)))
		.with_state(db)
}

#[instrument]
async fn index() -> Markup {
	templates::base(&templates::index(VERSION, GIT_VERSION))
}

#[instrument]
async fn quotes(
	State(db): State<SqlitePool>,
	Query(query): Query<QuotesQuery>,
) -> Result<Markup, AppError> {
	let from_date = query
		.from_date
		.ok_or_else(|| anyhow::anyhow!("no from_date"))
		.and_then(|d| d.parse().with_context(|| "parsing from_date"))
		.unwrap_or_default();
	let to_date = query
		.to_date
		.ok_or_else(|| anyhow::anyhow!("no to_date"))
		.and_then(|d| d.parse().with_context(|| "parsing to_date"))
		.unwrap_or_else(|_| chrono::Utc::now().naive_utc());

	let selected = query.user.map(UserId::new);
	let quotes = if let Some(user_id) = selected {
		quotes::get_for_user_id(&db, from_date, to_date, user_id).await?
	} else {
		quotes::get_all_between(&db, from_date, to_date).await?
	};

	Ok(templates::base(&templates::quotes(
		quotes,
		selected.map(UserId::get),
		&from_date.to_string(),
		&to_date.to_string(),
	)))
}

#[instrument]
async fn drunks(State(db): State<SqlitePool>) -> Result<Markup, AppError> {
	let mut drunks = drunks::get_all(&db).await?;
	drunks.sort_by_key(drunks::Drunk::score);

	let last_spill_days: i64 = sqlx::query_scalar!(
        r#"SELECT max(last_spill) AS "last_spill?: chrono::NaiveDateTime" FROM drunk WHERE last_spill IS NOT NULL LIMIT 1;"#
    )
    .fetch_one(&db)
    .await?
    .map(|t| (Local::now() - t.and_local_timezone(Local).unwrap()).num_days())
    .unwrap_or_default();

	Ok(templates::base(&templates::drunks(drunks, last_spill_days)))
}

#[instrument]
async fn not_found() -> impl IntoResponse {
	(StatusCode::NOT_FOUND, "404 ye pagrod")
}

// Make our own error that wraps `anyhow::Error`.
// Taken from https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			format!("Something went wrong: {}", self.0),
		)
			.into_response()
	}
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
	E: Into<anyhow::Error>,
{
	fn from(err: E) -> Self {
		Self(err.into())
	}
}
