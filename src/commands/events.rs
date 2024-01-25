use poise::serenity_prelude::{self as serenity, ReactionType};
use tracing::instrument;

pub async fn handler(
	ctx: &serenity::Context,
	event: &serenity::FullEvent,
	_framework: poise::FrameworkContext<'_, crate::Data, anyhow::Error>,
	_data: &crate::Data,
) -> anyhow::Result<()> {
	match event {
		serenity::FullEvent::ReactionAdd { add_reaction } => downvote(ctx, add_reaction).await,
		_ => Ok(()),
	}
}

const DOWN: &str = "⬇️";
const DOWNVOTE_LIMIT: u8 = 5;

#[instrument]
async fn downvote(ctx: &serenity::Context, reaction: &serenity::Reaction) -> anyhow::Result<()> {
	let react = &reaction.emoji;
	let message = ctx
		.http
		.get_message(reaction.channel_id, reaction.message_id)
		.await?;

	if let ReactionType::Unicode(ref emoji) = react {
		if emoji == DOWN {
			if let Some(r) = message
				.reactions
				.iter()
				.find(|&r| r.reaction_type == *react)
			{
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
