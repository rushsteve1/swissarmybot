use anyhow::anyhow;
use poise::serenity_prelude::{self as serenity, CreateMessage, Mentionable, ReactionType};
use rand::prelude::IndexedRandom;
use tracing::{info, instrument};

#[instrument]
pub async fn handler(
	ctx: &serenity::Context,
	event: &serenity::FullEvent,
	_: poise::FrameworkContext<'_, crate::Data, anyhow::Error>,
	_: &crate::Data,
) -> anyhow::Result<()> {
	match event {
		serenity::FullEvent::ReactionAdd { add_reaction } => {
			reaction_handler(ctx, add_reaction).await
		}
		serenity::FullEvent::VoiceStateUpdate { old, new } => {
			voice_state_handler(ctx, old, new).await
		}
		_ => Ok(()),
	}
}

const DOWN: &str = "⬇️";
const DOWNVOTE_LIMIT: u8 = 5;

#[instrument]
async fn reaction_handler(
	ctx: &serenity::Context,
	reaction: &serenity::Reaction,
) -> anyhow::Result<()> {
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

const CALLOUTS: &[&str] = &[
	"joined",
	"hopped in",
	"hopped on",
	"sidled over to",
	"wandered over to",
	"connected to",
	"hit the button on",
	"jumped into",
	"crashed",
	"landed on",
	"activated",
	"wants to hang in",
	"cordially invites you to",
	"has become a member of",
	"sat down at",
	"mounted",
	"awaits your presence in",
];

const RADON_ID: u64 = 741129884470083634;
const FRIENDSHIP_ID: u64 = 1395071978838167623;
async fn voice_state_handler(
	ctx: &serenity::Context,
	old: &Option<serenity::VoiceState>,
	new: &serenity::VoiceState,
) -> anyhow::Result<()> {
	// Don't repeatedly send messages
	if let Some(old) = old {
		if old.user_id == new.user_id {
			return Ok(());
		}
	}

	// Get the voice channel
	let channel = serenity::ChannelId::from(new.channel_id.ok_or(anyhow!("Channel ID not found"))?);
	let channels = new
		.guild_id
		.ok_or(anyhow!("Guild ID not found"))?
		.channels(ctx)
		.await?;
	let voice_channel = channels.get(&channel).ok_or(anyhow!("Channel not found"))?;

	// Only ping if someone joins an empty channel
	if voice_channel.members(ctx.cache.clone())?.len() > 1 {
		return Ok(());
	}

	let member = new.member.clone().ok_or(anyhow::anyhow!("No member"))?;

	// Ignore them if they join deafened
	if new.self_deaf {
		return Ok(());
	}

	// Send the message to the correct channel
	let radon = serenity::ChannelId::from(RADON_ID);
	let builder = CreateMessage::new().content(format!(
		"{} {} {} {}",
		serenity::RoleId::from(FRIENDSHIP_ID).mention(),
		member.mention(),
		CALLOUTS.choose(&mut rand::rng()).unwrap(),
		voice_channel.mention()
	));
	let msg = radon.send_message(ctx, builder).await?;

	// Sleep for 10 minutes then delete
	tokio::time::sleep(std::time::Duration::from_secs(10 * 60)).await;
	msg.delete(ctx).await?;

	Ok(())
}
