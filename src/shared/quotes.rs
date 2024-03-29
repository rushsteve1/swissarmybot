use anyhow::Context;
use chrono::NaiveDateTime;
use poise::serenity_prelude::UserId;
use sqlx::SqlitePool;
use tracing::instrument;

use super::helpers::to_userid;

#[derive(sqlx::FromRow)]
pub struct Quote {
	pub id: i64,
	pub user_id: String,
	pub user_name: String,
	pub author_id: String,
	pub author_name: String,
	pub text: String,
	pub inserted_at: NaiveDateTime,
}

#[instrument]
pub async fn add(
	db: &SqlitePool,
	user_id: UserId,
	user_name: &str,
	author_id: UserId,
	author_name: &str,
	text: &str,
) -> anyhow::Result<()> {
	let user_id_s = user_id.to_string();
	let author_id_s = author_id.to_string();

	sqlx::query!("INSERT INTO quotes (text, user_id, user_name, author_id, author_name) VALUES (?, ?, ?, ?, ?);",
                        text,
                        user_id_s, user_name, author_id_s, author_name)
                        .execute(db)
                        .await
                        .with_context(|| "error inserting quote")?;

	Ok(())
}

#[instrument]
pub async fn remove(db: &SqlitePool, id: i64) -> anyhow::Result<Option<UserId>> {
	sqlx::query_scalar!(
		r#"DELETE FROM quotes WHERE id = ? RETURNING user_id AS "CleverNum";"#,
		id
	)
	.fetch_optional(db)
	.await
	.map(|o| o.map(to_userid))
	.with_context(|| "error deleting quote")
}

#[instrument]
pub async fn get_one(db: &SqlitePool, id: i64) -> anyhow::Result<Option<Quote>> {
	sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = ?;", id)
		.fetch_optional(db)
		.await
		.with_context(|| "error getting quote")
}

#[instrument]
pub async fn get_all(db: &SqlitePool) -> anyhow::Result<Vec<Quote>> {
	get_all_between(db, NaiveDateTime::MIN, NaiveDateTime::MAX).await
}

#[instrument]
pub async fn get_all_between(
	db: &SqlitePool,
	from_date: chrono::NaiveDateTime,
	to_date: chrono::NaiveDateTime,
) -> anyhow::Result<Vec<Quote>> {
	sqlx::query_as!(
		Quote,
		"SELECT * FROM quotes WHERE inserted_at BETWEEN ? AND ?;",
		from_date,
		to_date
	)
	.fetch_all(db)
	.await
	.with_context(|| "getting quotes")
}

#[instrument]
pub async fn get_for_user_id(
	db: &SqlitePool,
	from_date: chrono::NaiveDateTime,
	to_date: chrono::NaiveDateTime,
	user_id: UserId,
) -> anyhow::Result<Vec<Quote>> {
	let user_id = user_id.to_string();
	sqlx::query_as!(
		Quote,
		"SELECT * FROM quotes WHERE user_id = ? AND inserted_at BETWEEN ? AND ?;",
		user_id,
		from_date,
		to_date
	)
	.fetch_all(db)
	.await
	.with_context(|| "getting quotes for user id")
}

#[instrument]
pub async fn get_random(db: &SqlitePool) -> anyhow::Result<Quote> {
	sqlx::query_as!(Quote, "SELECT * FROM quotes ORDER BY RANDOM() LIMIT 1;")
		.fetch_one(db)
		.await
		.with_context(|| "getting quote")
}

#[instrument]
pub fn list_url(cfg: &crate::Config, user_id: Option<UserId>) -> String {
	user_id
		.map(|u| format!("http://{}/quotes?user={}", cfg.domain, u))
		.unwrap_or(format!("http://{}/quotes", cfg.domain))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_list_url() {
		let cfg = crate::Config {
			domain: "sab.rushsteve1.us".to_string(),
			..Default::default()
		};

		assert_eq!("https://sab.rushsteve1.us/quotes", list_url(&cfg, None));
	}
}
