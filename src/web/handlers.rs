use std::fmt;
use std::str::FromStr;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::Local;
use serde::Deserializer;
use serde::{de, Deserialize};
use serenity::all::UserId;
use sqlx::SqlitePool;
use tracing::instrument;

use super::templates::*;

use crate::shared::{bigmoji, drunks, quotes};
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

pub fn router(db: SqlitePool) -> Router {
	Router::new()
		.route("/", get(index))
		.route("/bigmoji", get(bigmoji))
		.route("/drunks", get(drunks))
		.route("/quotes", get(quotes))
		.fallback(not_found)
		.with_state(db)
}

#[instrument]
async fn index() -> IndexTemplate {
	IndexTemplate {
		version: VERSION,
		git_version: GIT_VERSION,
	}
}

#[instrument]
async fn bigmoji(State(db): State<SqlitePool>) -> Result<BigMojiTemplate, AppError> {
	Ok(BigMojiTemplate {
		bigmoji: bigmoji::get_all(db).await?,
	})
}

#[instrument]
async fn quotes(
	State(db): State<SqlitePool>,
	Query(query): Query<QuotesQuery>,
) -> Result<QuotesTemplate, AppError> {
	let from_date = query
		.from_date
		.map(|d| d.parse().unwrap_or_default())
		.unwrap_or_default();
	let to_date = query
		.to_date
		.map(|d| d.parse().unwrap_or_default())
		.unwrap_or_default();

	let selected = query.user.map(UserId::new);
	let quotes = if let Some(user_id) = selected {
		quotes::get_for_user_id(db, from_date, to_date, user_id).await?
	} else {
		quotes::get_all(db, from_date, to_date).await?
	};

	Ok(QuotesTemplate {
		quotes,
		selected: selected.map(|u| u.get()),
		from_date: from_date.to_string(),
		to_date: to_date.to_string(),
	})
}

#[instrument]
async fn drunks(State(db): State<SqlitePool>) -> Result<DrunksTemplate, AppError> {
	let drunks = drunks::get_all(db.clone()).await?;

	let last_spill_days: i64 = sqlx::query_scalar!(
        r#"SELECT max(last_spill) AS "last_spill?: chrono::NaiveDateTime" FROM drunk WHERE last_spill IS NOT NULL LIMIT 1;"#
    )
    .fetch_one(&db)
    .await?
    .map(|t| (Local::now() - t.and_local_timezone(Local).unwrap()).num_days())
    .unwrap_or_default();

	Ok(DrunksTemplate {
		drunks,
		last_spill_days,
	})
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
