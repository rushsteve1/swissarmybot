use anyhow::Context;
use chrono::NaiveDateTime;
use poise::serenity_prelude::UserId;
use sqlx::SqlitePool;
use tracing::instrument;

#[derive(sqlx::FromRow)]
pub struct Drunk {
	pub id: i64,
	pub user_id: String,
	pub user_name: String,
	pub beer: i64,
	pub wine: i64,
	pub shots: i64,
	pub cocktails: i64,
	pub derby: i64,
	pub water: i64,
	pub last_drink: Option<String>,
	pub last_spill: Option<NaiveDateTime>,
	pub updated_at: NaiveDateTime,
}

impl Drunk {
	pub fn score(&self) -> i64 {
		(self.beer.clone() * 1)
			+ (self.wine.clone() * 2)
			+ (self.shots.clone() * 2)
			+ (self.cocktails.clone() * 2)
			+ (self.derby.clone() * 3)
	}
}

impl Drunk {
	pub fn last_spill_str(&self) -> String {
		self.last_spill.map_or("N/A".to_string(), |o| o.to_string())
	}
}

#[instrument]
pub async fn update(
	db: &SqlitePool,
	author_id: UserId,
	author_name: &str,
	drink_type: &str,
	drink_name: Option<&str>,
) -> anyhow::Result<(UserId, String)> {
	let author_id_s = author_id.to_string();

	let tx = db.begin().await?;

	sqlx::query!(
		"INSERT INTO drunk (user_id, user_name) VALUES (?, ?) ON CONFLICT (user_id) DO NOTHING;",
		author_id_s,
		author_name
	)
	.execute(db)
	.await
	.with_context(|| "inserting drunk")?;

	sqlx::query!(
		"UPDATE drunk SET updated_at = CURRENT_TIMESTAMP WHERE user_id = ?;",
		author_id_s
	)
	.execute(db)
	.await
	.with_context(|| "update drunk timestamp")?;

	// Repetitive, but that's the price of compile-time SQL validation
	let dr = match drink_type {
		"beer" => Some(sqlx::query!(
			"UPDATE drunk SET beer = beer + 1 WHERE user_id = ?;",
			author_id_s
		)),
		"wine" => Some(sqlx::query!(
			"UPDATE drunk SET wine = wine + 1 WHERE user_id = ?;",
			author_id_s
		)),
		"shot" => Some(sqlx::query!(
			"UPDATE drunk SET shots = shots + 1 WHERE user_id = ?;",
			author_id_s
		)),
		"cocktail" => Some(sqlx::query!(
			"UPDATE drunk SET cocktails = cocktails + 1 WHERE user_id = ?;",
			author_id_s
		)),
		"derby" => Some(sqlx::query!(
			"UPDATE drunk SET derby = derby + 1 WHERE user_id = ?;",
			author_id_s
		)),
		"water" => Some(sqlx::query!(
			"UPDATE drunk SET water = water + 1 WHERE user_id = ?;",
			author_id_s
		)),
		_ => None,
	};

	if let Some(dq) = dr {
		dq.execute(db).await.with_context(|| "updating drunk")?;
	}

	let type_str = drink_name.map_or_else(
		|| drink_type.to_string(),
		|name| format!("{drink_type}: {name}"),
	);

	sqlx::query!(
		"UPDATE drunk SET last_drink = ? WHERE user_id = ?;",
		type_str,
		author_id_s
	)
	.execute(db)
	.await
	.with_context(|| "updating last_drink")?;

	tx.commit().await?;

	Ok((author_id, type_str))
}

#[instrument]
pub async fn get_all(db: &SqlitePool) -> anyhow::Result<Vec<Drunk>> {
	sqlx::query_as!(Drunk, "SELECT * FROM drunk;")
		.fetch_all(db)
		.await
		.with_context(|| "getting drunks")
}
