use anyhow::Context;
use poise::serenity_prelude::{self as serenity, CreateEmbed, Member, Mentionable, Message};
use sqlx::PgPool;
use tracing::instrument;

use crate::shared::helpers::to_userid;
use crate::shared::quotes::{self, get_page, get_page_count};
use crate::Ctx;

/// Manage peoples' quotes
#[poise::command(
	slash_command,
	rename = "quote",
	subcommands("add", "remove", "get", "list"),
	subcommand_required
)]
#[allow(clippy::unused_async)]
pub async fn top<E>(_ctx: Ctx<'_>) -> Result<(), E> {
	Ok(())
}

/// Add a quote to the database
#[instrument]
#[poise::command(slash_command)]
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
#[instrument]
#[poise::command(slash_command)]
async fn remove(
	ctx: Ctx<'_>,
	#[description = "What number quote should be removed?"] number: i32,
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
#[instrument]
#[poise::command(slash_command)]
async fn get(
	ctx: Ctx<'_>,
	#[description = "What number quote should be gotten?"] number: i32,
) -> anyhow::Result<()> {
	let quote = quotes::get_one(&ctx.data().db, number).await?;

	let reply = quote
		.map(|q| {
			format!(
				"Quote {} by {}\n>>> {}",
				number,
				to_userid(q.user_id).mention(),
				q.quote
			)
		})
		.unwrap_or(format!("Quote {number} does not exist"));

	ctx.say(reply).await.with_context(|| "quote get reply")?;

	Ok(())
}

/// List all the quotes by this user (or everyone)
#[instrument]
#[poise::command(slash_command)]
async fn list(
	ctx: Ctx<'_>,
	#[description = "Who is this quote by?"] user: Option<Member>,
) -> anyhow::Result<()> {
	let user = user.unwrap();
	let count = get_page_count(&ctx.data().db, user.user.id).await?;

	// Adapted from: https://docs.rs/poise/latest/src/poise/builtins/paginate.rs.html#35-94

	// Define some unique identifiers for the navigation buttons
	let ctx_id = ctx.id();
	let prev_button_id = format!("{}prev", ctx_id);
	let next_button_id = format!("{}next", ctx_id);

	// Send the embed with the first page as content
	let reply = {
		let components = serenity::CreateActionRow::Buttons(vec![
			serenity::CreateButton::new(&prev_button_id).emoji('◀'),
			serenity::CreateButton::new(&next_button_id).emoji('▶'),
		]);

		poise::CreateReply::default()
			.embed(quotes_embed(&ctx.data().db, &user, 0).await?)
			.components(vec![components])
	};

	ctx.send(reply).await?;

	// Loop through incoming interactions with the navigation buttons
	let mut current_page: i32 = 0;
	while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
		// We defined our button IDs to start with `ctx_id`. If they don't, some other command's
		// button was pressed
		.filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
		// Timeout when no navigation button has been pressed for 24 hours
		.timeout(std::time::Duration::from_secs(3600 * 24))
		.await
	{
		// Depending on which button was pressed, go to next or previous page
		if press.data.custom_id == next_button_id {
			current_page += 1;
			if current_page >= count {
				current_page = 0;
			}
		} else if press.data.custom_id == prev_button_id {
			current_page = current_page.checked_sub(1).unwrap_or(count - 1);
		} else {
			// This is an unrelated button interaction
			continue;
		}

		// Update the message with the new page contents
		press
			.create_response(
				ctx.serenity_context(),
				serenity::CreateInteractionResponse::UpdateMessage(
					serenity::CreateInteractionResponseMessage::new()
						.embed(quotes_embed(&ctx.data().db, &user, current_page).await?),
				),
			)
			.await?;
	}

	Ok(())
}

#[instrument]
#[poise::command(context_menu_command = "Add Quote")]
pub async fn context_menu(
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

async fn quotes_embed(db: &PgPool, user: &Member, page: i32) -> anyhow::Result<CreateEmbed> {
	let quotes = get_page(db, user.user.id, page).await?;

	let embed = serenity::CreateEmbed::default()
		.fields(quotes.iter().map(|q| (q.id.to_string(), &q.quote, true)))
		.author(serenity::CreateEmbedAuthor::from(&user.user));

	Ok(embed)
}
