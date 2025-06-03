use anyhow::{anyhow, Context};
use poise::serenity_prelude::{self as serenity, CreateEmbed, Member, Message};
use sqlx::PgPool;
use tracing::{info, instrument};

use crate::quotes::{self, get_page, get_quote_count, to_userid, Quote, PAGE_SIZE};
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
	let number = quotes::add(
		&ctx.data().db,
		user.user.id,
		&user.user.name,
		ctx.author().id,
		&ctx.author().name,
		&text,
	)
	.await?;

	let quote = quotes::get_one(&ctx.data().db, number)
		.await?
		.ok_or_else(|| anyhow!("quote not found"))?;
	let reply = poise::CreateReply::default().embed(quote.embed(ctx).await?);

	return ctx
		.send(reply)
		.await
		.map(|_| ())
		.with_context(|| "quote add reply");
}

/// Remove a quote from the database
#[instrument]
#[poise::command(slash_command)]
async fn remove(
	ctx: Ctx<'_>,
	#[description = "What number quote should be removed?"] number: i32,
) -> anyhow::Result<()> {
	let _row = quotes::remove(&ctx.data().db, number).await?;

	return ctx
		.say(format!("Quote {number} removed"))
		.await
		.map(|_| ())
		.with_context(|| "quote remove reply");
}

/// Get a quote from the database
#[instrument]
#[poise::command(slash_command)]
async fn get(
	ctx: Ctx<'_>,
	#[description = "What number quote should be gotten?"] number: i32,
) -> anyhow::Result<()> {
	let quote = quotes::get_one(&ctx.data().db, number)
		.await?
		.ok_or_else(|| anyhow!("quote not found"))?;
	let reply = poise::CreateReply::default().embed(quote.embed(ctx).await?);

	return ctx
		.send(reply)
		.await
		.map(|_| ())
		.with_context(|| "quote get reply");
}

/// List all the quotes by a user
#[instrument]
#[poise::command(slash_command)]
async fn list(
	ctx: Ctx<'_>,
	#[description = "Who is this quote by?"] user: Member,
) -> anyhow::Result<()> {
	let count = get_quote_count(&ctx.data().db, user.user.id).await?;

	if count == 0 {
		return ctx
			.reply("This user has no quotes!")
			.await
			.map(|_| ())
			.with_context(|| "no quotes");
	}

	let page_len = (count / PAGE_SIZE) + 1;

	// Adapted from: https://docs.rs/poise/latest/src/poise/builtins/paginate.rs.html#35-94

	// Define some unique identifiers for the navigation buttons
	let ctx_id = ctx.id();
	let prev_button_id = format!("{ctx_id}prev");
	let next_button_id = format!("{ctx_id}next");

	info!("list command {} started", ctx_id);

	// Send the embed with the first page as content
	let reply = {
		let components = serenity::CreateActionRow::Buttons(vec![
			serenity::CreateButton::new(&prev_button_id).emoji('◀'),
			serenity::CreateButton::new(&next_button_id).emoji('▶'),
		]);

		poise::CreateReply::default()
			.embed(quotes_embed(&ctx.data().db, &user, 0, page_len).await?)
			.components(vec![components])
	};

	let msg = ctx.send(reply).await?;

	// Loop through incoming interactions with the navigation buttons
	let mut current_page: usize = 0;
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
			if current_page >= page_len {
				current_page = 0;
			}
		} else if press.data.custom_id == prev_button_id {
			current_page = current_page.checked_sub(1).unwrap_or(page_len - 1);
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
						.embed(quotes_embed(&ctx.data().db, &user, current_page, page_len).await?),
				),
			)
			.await?;
	}

	info!("list command {} ended", ctx_id);

	let reply = poise::CreateReply::default()
		.embed(serenity::CreateEmbed::new().description("Interaction ended"));
	return msg
		.edit(ctx, reply)
		.await
		.with_context(|| "interaction ended reply");
}

#[instrument]
#[poise::command(context_menu_command = "Add Quote")]
pub async fn context_menu(
	ctx: Ctx<'_>,
	#[description = "The message to add as a quote"] msg: Message,
) -> anyhow::Result<()> {
	let number = quotes::add(
		&ctx.data().db,
		msg.author.id,
		&msg.author.name,
		ctx.author().id,
		&ctx.author().name,
		&msg.content,
	)
	.await?;

	let quote = quotes::get_one(&ctx.data().db, number)
		.await?
		.ok_or_else(|| anyhow!("quote not found"))?;
	let reply = poise::CreateReply::default().embed(quote.embed(ctx).await?);

	return ctx
		.send(reply)
		.await
		.map(|_| ())
		.with_context(|| "quote add reply");
}

impl Quote {
	async fn embed(self, ctx: Ctx<'_>) -> anyhow::Result<CreateEmbed> {
		let user = ctx.http().get_user(to_userid(&self.user_id)).await?;
		let author = ctx.http().get_user(to_userid(&self.author_id)).await?;

		Ok(serenity::CreateEmbed::new()
			.title(format!("Quote #{}", self.id))
			.description(self.quote_trunc())
			.footer(
				serenity::CreateEmbedFooter::new(format!(
					"Added by {} on {}",
					author.display_name(),
					self.created_at
				))
				.icon_url(author.avatar_url().unwrap_or_default()),
			)
			.author(serenity::CreateEmbedAuthor::from(user)))
	}
}

#[instrument]
async fn quotes_embed(
	db: &PgPool,
	user: &Member,
	page: usize,
	len: usize,
) -> anyhow::Result<CreateEmbed> {
	let quotes = get_page(db, user.user.id, page).await?;

	let embed = serenity::CreateEmbed::default()
		.fields(quotes.iter().map(|q|
			(format!("Quote #{}", q.id), q.quote_trunc(), false)
		))
		.footer(serenity::CreateEmbedFooter::new(format!(
			"Page {} of {}",
			page + 1,
			len
		)))
		.author(serenity::CreateEmbedAuthor::from(&user.user));

	Ok(embed)
}
