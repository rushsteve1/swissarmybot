use anyhow::Context;
use chrono::NaiveDateTime;
use poise::serenity_prelude::UserId;
use sqlx::PgPool;
use tracing::instrument;

pub fn to_userid(s: impl Into<String>) -> UserId {
	s.into().parse().unwrap_or_default()
}

#[derive(sqlx::FromRow)]
pub struct Quote {
	pub id: i64,
	pub user_id: String,
	pub author_id: String,
	#[allow(clippy::struct_field_names)]
	pub quote: String,
	pub created_at: NaiveDateTime,
	pub ext: Option<serde_json::Value>,
}

impl Quote {
    pub fn quote_trunc(&self) -> &str {
        // Discord's limit is 1024 so this gives us a little room
        let l = std::cmp::min(self.quote.len(), 1000);
        &self.quote[..l]
    }
}

#[instrument]
pub async fn create_table(db: &PgPool) -> anyhow::Result<()> {
	return sqlx::query!(
		"CREATE TABLE IF NOT EXISTS quotes (
			id SERIAL PRIMARY KEY,
			user_id text NOT NULL,
			author_id text NOT NULL,
			quote text NOT NULL,
			created_at timestamp NOT NULL DEFAULT now(),
			ext jsonb
		);"
	)
	.execute(db)
	.await
	.map(|_| ())
	.with_context(|| "error creating table");
}

#[instrument]
pub async fn add(
	db: &PgPool,
	user_id: UserId,
	user_name: &str,
	author_id: UserId,
	author_name: &str,
	text: &str,
) -> anyhow::Result<i32> {
	let user_id_s = user_id.to_string();
	let author_id_s = author_id.to_string();

	return sqlx::query_scalar!(
		"INSERT INTO quotes (quote, user_id, author_id) VALUES ($1, $2, $3) RETURNING id;",
		text,
		user_id_s,
		author_id_s
	)
	.fetch_one(db)
	.await
	.with_context(|| "error inserting quote");
}

#[instrument]
pub async fn remove(db: &PgPool, id: i32) -> anyhow::Result<Option<UserId>> {
	sqlx::query_scalar!(
		r#"DELETE FROM quotes WHERE id = $1 RETURNING user_id AS "CleverNum";"#,
		id
	)
	.fetch_optional(db)
	.await
	.map(|o| o.map(to_userid))
	.with_context(|| "error deleting quote")
}

#[instrument]
pub async fn get_one(db: &PgPool, id: i32) -> anyhow::Result<Option<Quote>> {
	sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = $1;", id)
		.fetch_optional(db)
		.await
		.with_context(|| "error getting quote")
}

#[instrument]
pub async fn get_all(db: &PgPool) -> anyhow::Result<Vec<Quote>> {
	get_all_between(db, NaiveDateTime::MIN, NaiveDateTime::MAX).await
}

#[instrument]
pub async fn get_all_between(
	db: &PgPool,
	from_date: chrono::NaiveDateTime,
	to_date: chrono::NaiveDateTime,
) -> anyhow::Result<Vec<Quote>> {
	sqlx::query_as!(
		Quote,
		"SELECT * FROM quotes WHERE created_at BETWEEN $1 AND $2;",
		from_date,
		to_date
	)
	.fetch_all(db)
	.await
	.with_context(|| "getting quotes")
}

#[instrument]
pub async fn get_for_user_id(
	db: &PgPool,
	from_date: chrono::NaiveDateTime,
	to_date: chrono::NaiveDateTime,
	user_id: UserId,
) -> anyhow::Result<Vec<Quote>> {
	let user_id = user_id.to_string();
	sqlx::query_as!(
		Quote,
		"SELECT * FROM quotes WHERE user_id = $1 AND created_at BETWEEN $2 AND $3;",
		user_id,
		from_date,
		to_date
	)
	.fetch_all(db)
	.await
	.with_context(|| "getting quotes for user id")
}

#[instrument]
pub async fn get_random(db: &PgPool) -> anyhow::Result<Quote> {
	sqlx::query_as!(Quote, "SELECT * FROM quotes ORDER BY RANDOM() LIMIT 1;")
		.fetch_one(db)
		.await
		.with_context(|| "getting quote")
}

pub const PAGE_SIZE: usize = 5;

#[instrument]
pub async fn get_page(db: &PgPool, user_id: UserId, page: usize) -> anyhow::Result<Vec<Quote>> {
	let user_id = user_id.to_string();
	sqlx::query_as!(
		Quote,
		"SELECT * FROM quotes WHERE user_id = $1 LIMIT $2 OFFSET $3;",
		user_id,
		PAGE_SIZE as i64,
		(page * PAGE_SIZE) as i64
	)
	.fetch_all(db)
	.await
	.with_context(|| "getting quotes page")
}

#[instrument]
pub async fn get_quote_count(db: &PgPool, user_id: UserId) -> anyhow::Result<usize> {
	let user_id = user_id.to_string();
	let count = sqlx::query_scalar!("SELECT COUNT(*) FROM quotes WHERE user_id = $1;", user_id)
		.fetch_one(db)
		.await
		.with_context(|| "getting page count")?;

	Ok(count.unwrap_or(0) as usize)
}
