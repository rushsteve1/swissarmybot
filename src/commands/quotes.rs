use anyhow::Context;
use poise::serenity_prelude::{Member, Mentionable, Message, UserId};
use tracing::instrument;

use crate::shared::quotes;
use crate::Ctx;

/// Manage peoples' quotes
#[poise::command(
	slash_command,
	rename = "quotes",
	subcommands("add", "remove", "get", "list"),
	subcommand_required
)]
#[allow(clippy::unused_async)]
pub async fn top(_ctx: Ctx<'_>) -> anyhow::Result<()> {
	Ok(())
}

/// Add a quote to the database
#[poise::command(slash_command)]
#[instrument]
async fn add(
	ctx: Ctx<'_>,
	#[description = "Who is this quote by?"] user: Member,
	#[description = "What did they say?"] text: String,
) -> anyhow::Result<()> {
	quotes::add(
		&ctx.data().db,
		user.user.id,
		&user.user.name,
		ctx.author().id,
		&ctx.author().name,
		&text,
	)
	.await?;

	ctx.say(format!("Quote added for {}\n>>> {}", user.mention(), text))
		.await?;

	Ok(())
}

/// Remove a quote from the database
#[poise::command(slash_command)]
#[instrument]
async fn remove(
	ctx: Ctx<'_>,
	#[description = "What number quote should be removed?"] number: i64,
) -> anyhow::Result<()> {
	let row = quotes::remove(&ctx.data().db, number).await?;

	ctx.say(
		row.map(|user_id| format!("Quote {} removed by {}", number, user_id.mention()))
			.unwrap_or(format!("Quote {number} does not exist")),
	)
	.await?;

	Ok(())
}

/// Get a quote from the database
#[poise::command(slash_command)]
#[instrument]
async fn get(
	ctx: Ctx<'_>,
	#[description = "What number quote should be gotten?"] number: i64,
) -> anyhow::Result<()> {
	let quote = quotes::get_one(&ctx.data().db, number).await?;

	let reply = quote
		.map(|q| {
			format!(
				"Quote {} by {}\n>>> {}",
				number,
				q.user_id.parse::<UserId>().unwrap_or_default().mention(),
				q.text
			)
		})
		.unwrap_or(format!("Quote {number} does not exist"));

	ctx.say(reply).await.with_context(|| "quote get reply")?;

	Ok(())
}

/// List all the quotes by this user (or everyone)
#[poise::command(slash_command)]
#[instrument]
async fn list(
	ctx: Ctx<'_>,
	#[description = "Who is this quote by?"] user: Option<Member>,
) -> anyhow::Result<()> {
	ctx.say(quotes::list_url(&ctx.data().cfg, user.map(|u| u.user.id)))
		.await
		.with_context(|| "quote list reply")?;

	Ok(())
}

#[poise::command(context_menu_command = "Add Quote")]
#[instrument]
async fn context_menu(
	ctx: Ctx<'_>,
	#[description = "The message to add as a quote"] msg: Message,
) -> anyhow::Result<()> {
	let text = &msg.content_safe(ctx.cache());
	quotes::add(
		&ctx.data().db,
		msg.author.id,
		&msg.author.name,
		ctx.author().id,
		&ctx.author().name,
		text,
	)
	.await?;

	ctx.say(format!(
		"Quote added for {}\n>>> {}",
		msg.author.mention(),
		text
	))
	.await?;

	Ok(())
}
