use anyhow::Context;
use serenity::all::{
    CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, Interaction,
    Message, Reaction, ReactionType, Ready,
};
use serenity::all::Context as Ctx;
use serenity::async_trait;
use tracing::{error, info, instrument};

use super::definition::interactions_definition;
use crate::commands::bigmoji::BigMoji;
use crate::helpers::{get_bigmoji, get_cmd, get_inter};
use crate::{DB_POOL, DOMAIN, PREFIX};

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
        let _commands = interactions_definition(ctx).await;
    }

    #[instrument]
    async fn interaction_create(&self, ctx: Ctx, interaction: Interaction) {
        let Ok(inter) = get_inter(&interaction) else {
            error!("slash command had no data");
            return;
        };

        let res = match inter.data.name.as_str() {
            "quote" => handle_quote_command(&interaction).await,
            "bigmoji" => handle_bigmoji_command(&interaction).await,
            "drunk" => handle_drunk_command(&interaction).await,
            _ => Err(anyhow::anyhow!("unknown command")),
        };

        let content = res.unwrap_or_else(|e| {
            error!(error = %e, "Error handling command");
            return "Ya broke it".to_string();
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
    async fn message(&self, ctx: Ctx, message: Message) {
        // Don't bother with bot messages (including our own)
        if message.author.bot {
            return;
        }

        send_bigmoji(ctx, &message).await.unwrap_or_else(|e| {
            error!(error = %e, "could not send bigmoji")
        });
    }

    #[instrument]
    async fn reaction_add(&self, ctx: serenity::all::Context, reaction: Reaction) {
        let message = ctx
            .http
            .get_message(reaction.channel_id, reaction.message_id)
            .await
            .unwrap();
        let react = reaction.emoji;

        if let ReactionType::Unicode(ref emoji) = react {
            if emoji == DOWN {
                let count = message
                    .reactions
                    .iter()
                    .find(|&r| r.reaction_type == react)
                    .unwrap()
                    .count;
                if count >= DOWNVOTE_LIMIT.into() {
                    message
                        .reply_ping(&ctx.http, "Message deleted, get fucked.")
                        .await
                        .unwrap();
                    message.delete(&ctx.http).await.unwrap();
                }
            }
        }
    }
}

#[instrument]
async fn handle_quote_command(interaction: &Interaction) -> anyhow::Result<String> {
    let cmd = get_cmd(interaction)?;

    match cmd.name.as_str() {
        "add" => super::quotes::add(interaction).await,
        "remove" => super::quotes::remove(interaction).await,
        "get" => super::quotes::get(interaction).await,
        "list" => super::quotes::list(interaction).await,
        _ => Err(anyhow::anyhow!("unknown quote command")),
    }
}

#[instrument]
async fn handle_bigmoji_command(interaction: &Interaction) -> anyhow::Result<String> {
    let cmd = get_cmd(interaction)?;

    match cmd.name.as_str() {
        "add" => super::bigmoji::add(interaction).await,
        "remove" => super::bigmoji::remove(interaction).await,
        "get" => super::bigmoji::get(interaction).await,
        "list" => Ok(format!("http://{}{}/bigmoji", *DOMAIN, *PREFIX)),
        _ => Err(anyhow::anyhow!("unknown bigmoji command")),
    }
}

#[instrument]
async fn handle_drunk_command(interaction: &Interaction) -> anyhow::Result<String> {
    super::drunk::update(interaction).await
}

#[instrument]
async fn send_bigmoji(ctx: Ctx, message: &Message) -> anyhow::Result<()> {
    let Ok(re) = regex::Regex::new(r":(\S+):") else {
        error!("could not compile regex");
        anyhow::bail!("could not compile regex")
    };

    for mat in re.captures_iter(&message.content) {
        let term = mat.get(1).unwrap().as_str().to_lowercase();
        let moji = get_bigmoji(term).await?;

        if let Some(moji) = moji {
            message
                .reply(&ctx.http, moji.text)
                .await
                .with_context(|| "responding to BigMoji")?;
        }
    }

    Ok(())
}
