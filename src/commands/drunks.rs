use poise::serenity_prelude::{Mentionable, UserId};
use tracing::instrument;

use crate::Ctx;

/// Record your tipsy times
#[poise::command(slash_command)]
#[instrument]
pub async fn drunk(
	ctx: Ctx<'_>,
	#[rename = "type"]
	#[description = "What kinda drink ya havin?"]
	#[autocomplete = "autocomplete_drink_type"]
	drink_type: String,
	#[rename = "name"]
	#[description = "Be more specific"]
	drink_name: Option<String>,
) -> anyhow::Result<()> {
	let (author_id, type_str) = crate::shared::drunks::update(
		&ctx.data().db,
		ctx.author().id,
		&ctx.author().name,
		&drink_type,
		drink_name.as_deref(),
	)
	.await?;

	ctx.say(format!("{} had a {}", author_id.mention(), type_str))
		.await?;

	Ok(())
}

const THE_CAPTAIN: UserId = UserId::new(115_178_518_391_947_265);

/// Report that a Spill has occured and you are the culprit
#[poise::command(slash_command)]
#[instrument]
pub async fn spill(ctx: Ctx<'_>) -> anyhow::Result<()> {
	let user_id = ctx.author().id.to_string();

	sqlx::query!(
		"UPDATE drunk SET last_spill = CURRENT_TIMESTAMP WHERE user_id = ?;",
		user_id,
	)
	.execute(&ctx.data().db)
	.await?;

	ctx.say(format!(
        "# SPILL ALERT\n{} **HAS SPILLED**\n**INFORMING THE COMMANDING OFFICER** {}\n\nThis incident has been recorded.",
        ctx.author().mention(),
        THE_CAPTAIN.mention()
    )).await?;

	Ok(())
}

const DRINK_TYPES: &[&str] = &["beer", "wine", "shot", "cocktail", "derby", "water"];

#[allow(clippy::unused_async)]
async fn autocomplete_drink_type<'a>(
	_ctx: Ctx<'_>,
	partial: &'a str,
) -> impl Iterator<Item = String> + 'a {
	DRINK_TYPES
		.iter()
		.filter(move |t| t.starts_with(partial))
		.map(ToString::to_string)
}
