use serenity::all::{
	Context as Ctx, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler,
	Interaction, Mentionable, Reaction, ReactionType, Ready,
};
use serenity::async_trait;
use tracing::{error, info, instrument};

use super::definition::interactions_definition;
use crate::shared::helpers::{get_cmd, get_db, get_inter, THE_CAPTAIN};

const DOWN: &str = "⬇️";
const DOWNVOTE_LIMIT: u8 = 5;

#[derive(Debug)]
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
	#[instrument]
	async fn ready(&self, ctx: Ctx, _ready: Ready) {
		info!("SwissArmyBot is ready!");

		// Upserts the existing commands
		interactions_definition(ctx).await.unwrap_or_else(|e| {
			error!(error = %e, "could not submit interaction defitions");
			Vec::new()
		});
	}

	#[instrument]
	async fn interaction_create(&self, ctx: Ctx, interaction: Interaction) {
		let Ok(inter) = get_inter(&interaction) else {
			error!("slash command had no data");
			return;
		};

		let res = match inter.data.name.as_str() {
			"quote" => handle_quote_command(ctx.clone(), &interaction).await,
			"drunk" => handle_drunk_command(ctx.clone(), &interaction).await,
			"spill" => handle_spill_command(ctx.clone(), &interaction).await,
			_ => Err(anyhow::anyhow!("unknown command")),
		};

		let content = res.unwrap_or_else(|e| {
			error!(error = %e, "Error handling command");
			"Ya broke it".to_string()
		});

		let res = inter
			.create_response(
				ctx,
				CreateInteractionResponse::Message(
					CreateInteractionResponseMessage::new().content(content),
				),
			)
			.await;

		if res.is_err() {
			error!(error = ?res, "Error responding to interaction");
		}
	}

	#[instrument]
	async fn reaction_add(&self, ctx: Ctx, reaction: Reaction) {
		handle_downvote(ctx, reaction)
			.await
			.unwrap_or_else(|e| error!(error = %e, "could not delete downvoted message"));
	}
}

#[instrument]
async fn handle_quote_command(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
	let cmd = get_cmd(interaction)?;

	match cmd.name.as_str() {
		"add" => super::quotes::add(ctx, interaction).await,
		"remove" => super::quotes::remove(ctx, interaction).await,
		"get" => super::quotes::get(ctx, interaction).await,
		"list" => super::quotes::list(ctx, interaction).await,
		_ => Err(anyhow::anyhow!("unknown quote command")),
	}
}

#[instrument]
async fn handle_drunk_command(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
	let db = get_db(ctx).await?;
	super::drunks::update(db, interaction).await
}

#[instrument]
async fn handle_downvote(ctx: Ctx, reaction: Reaction) -> anyhow::Result<()> {
	let react = reaction.emoji;
	let message = ctx
		.http
		.get_message(reaction.channel_id, reaction.message_id)
		.await?;

	if let ReactionType::Unicode(ref emoji) = react {
		if emoji == DOWN {
			if let Some(r) = message.reactions.iter().find(|&r| r.reaction_type == react) {
				if r.count >= DOWNVOTE_LIMIT.into() {
					message
						.reply_ping(ctx.clone(), "Message deleted, get fucked.")
						.await?;
					message.delete(ctx).await?;
				}
			}
		}
	}

	Ok(())
}

#[instrument]
async fn handle_spill_command(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
	let db = get_db(ctx).await?;
	let inter = get_inter(interaction)?;
	let author = inter
		.member
		.as_ref()
		.ok_or_else(|| anyhow::anyhow!("interaction had no author"))?;
	let author_id = author.user.id.to_string();

	sqlx::query!(
		"UPDATE drunk SET last_spill = CURRENT_TIMESTAMP WHERE user_id = ?;",
		author_id
	)
	.execute(&db)
	.await?;

	Ok(format!(
        "# SPILL ALERT\n{} **HAS SPILLED**\n**INFORMING THE COMMANDING OFFICER** {}\n\nThis incident has been recorded.",
        author.mention(),
        THE_CAPTAIN.mention()
    ))
}
