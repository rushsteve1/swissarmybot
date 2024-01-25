use anyhow::Context;
use chrono::NaiveDateTime;
use serenity::all::UserId;
use sqlx::SqlitePool;
use tracing::{error, instrument};

#[derive(sqlx::FromRow)]
pub struct Drunk {
	pub id: i64,
	pub user_id: i64,
	pub user_name: String,
	pub beer: i64,
	pub wine: i64,
	pub shots: i64,
	pub cocktails: i64,
	pub derby: i64,
	pub water: i64,
	pub updated_at: NaiveDateTime,
	pub score: i64,
	pub last_drink: Option<String>,
	pub last_spill: Option<NaiveDateTime>,
}

impl Drunk {
	pub fn last_spill_str(&self) -> String {
		self.last_spill.map_or("N/A".to_string(), |o| o.to_string())
	}
}

#[instrument]
pub async fn update(
	db: SqlitePool,
	author_id: UserId,
	author_name: &str,
	drink_type: &str,
	drink_name: Option<&str>,
) -> anyhow::Result<(UserId, String)> {
	let author_id_s = author_id.to_string();

	sqlx::query!(
		"INSERT INTO drunk (user_id, user_name) VALUES (?, ?) ON CONFLICT (user_id) DO NOTHING;",
		author_id_s,
		author_name
	)
	.execute(&db)
	.await
	.with_context(|| "inserting drunk")?;

	// Repetitive, but that's the price of compile-time SQL validation
	match drink_type {
            "beer" => sqlx::query!(
                "UPDATE drunk SET beer = beer + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "wine" => sqlx::query!(
                "UPDATE drunk SET wine = wine + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "shot" => sqlx::query!(
                "UPDATE drunk SET shots = shots + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "cocktail" => sqlx::query!(
                "UPDATE drunk SET cocktails = cocktails + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "derby" => sqlx::query!(
                "UPDATE drunk SET derby = derby + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            "water" => sqlx::query!(
                "UPDATE drunk SET water = water + 1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
                author_id_s
            ),
            _ => {
                error!("unknown drink type");
                return Err(anyhow::anyhow!("unknown drink type"));
            },
        }
        .execute(&db)
        .await
        .with_context(|| "updating drunk")?;

	let type_str = drink_name.map_or_else(
		|| drink_type.to_string(),
		|name| format!("{drink_type}: {name}"),
	);

	sqlx::query!(
		"UPDATE drunk SET last_drink = ? WHERE user_id = ?;",
		type_str,
		author_id_s
	)
	.execute(&db)
	.await
	.with_context(|| "updating last_drink")?;

	Ok((author_id, type_str))
}

#[instrument]
pub async fn get_all(db: SqlitePool) -> anyhow::Result<Vec<Drunk>> {
	sqlx::query_as!(Drunk, "SELECT * FROM drunk ORDER BY score DESC;")
		.fetch_all(&db)
		.await
		.with_context(|| "getting drunks")
}
