use anyhow::anyhow;
use serenity::all::Interaction;
use sqlx::SqlitePool;

use crate::shared::bigmoji;
use crate::shared::helpers::get_cmd;

pub async fn add(db: SqlitePool, interaction: &Interaction) -> anyhow::Result<String> {
	let cmd = get_cmd(interaction)?;

	let mut name = cmd.name.replace(':', "").to_lowercase();
	name.retain(|c| !c.is_whitespace());

	if name.len() < 3 {
		return Ok("BigMoji name too short".to_string());
	}

	let text = cmd.value.as_str().ok_or_else(|| anyhow!("bigmoji text"))?;

	bigmoji::add(db, name.as_str(), text).await?;

	Ok(format!("BigMoji `:{name}:` added"))
}

pub async fn remove(db: SqlitePool, interaction: &Interaction) -> anyhow::Result<String> {
	let cmd = get_cmd(interaction)?;
	let mut name = cmd.name.replace(':', "").to_lowercase();
	name.retain(|c| !c.is_whitespace());

	bigmoji::remove(db, name.as_str()).await?;

	Ok(format!("Deleted BigMoji `:{name}:`"))
}

pub async fn get(db: SqlitePool, interaction: &Interaction) -> anyhow::Result<String> {
	let cmd = get_cmd(interaction)?;

	let mut name = cmd.name.replace(':', "").to_lowercase();
	name.retain(|c| !c.is_whitespace());

	let moji = bigmoji::get_one(db, name.as_str()).await?;

	Ok(moji
		.map(|m| m.text)
		.unwrap_or(format!("BigMoji `:{name}:` does not exist")))
}
