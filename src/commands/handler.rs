use serenity::async_trait;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::*;
use serenity::prelude::*;

use super::definition::{clear_definitions, clear_definitions_for_guild, interactions_definition};
use crate::models::BigMoji;
use crate::{DB_POOL, DOMAIN, PREFIX};

const DOWN: &str = "⬇️";
const DOWNVOTE_LIMIT: u8 = 5;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("SwissArmyBot is ready!");

        let clear: bool = std::option_env!("CLEAR_DEFINITIONS")
            .unwrap_or("false")
            .parse()
            .unwrap();

        if clear {
            warn!("Clearing slash command definitions");
            clear_definitions(&ctx).await;

            for guild in ready.guilds {
                if guild.unavailable {
                    clear_definitions_for_guild(&ctx, guild.id).await;
                }
            }
        }

        let _commands = interactions_definition(ctx).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(ref inter) = interaction {
            let content = match inter.data.name.as_str() {
                "quote" => handle_quote_command(&interaction).await,
                "bigmoji" => handle_bigmoji_command(&interaction).await,
                _ => "Unknown Command".to_string(),
            };

            if let Err(e) = inter
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                error!("Error responding to slash command {:?}", e);
            }
        } else {
            warn!("Slash command interaction had no data {:?}", interaction);
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        // Don't bother with bot messages (including our own)
        if message.author.bot {
            return;
        }

        let re = regex::Regex::new(r":(\S+):").unwrap();

        for mat in re.captures_iter(&message.content) {
            let term = mat.get(1).unwrap().as_str().to_lowercase();
            let moji: Option<BigMoji> = sqlx::query_as("SELECT * FROM bigmoji WHERE name = ?;")
                .bind(term)
                .fetch_optional(&*DB_POOL)
                .await
                .expect("Error getting BigMoji in message");

            if let Some(moji) = moji {
                message
                    .reply(&ctx.http, moji.text)
                    .await
                    .expect("Error responding to BigMoji");
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let message = ctx
            .http
            .get_message(reaction.channel_id.into(), reaction.message_id.into())
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

async fn handle_quote_command(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    match cmd.name.as_str() {
        "add" => super::quotes::add(interaction).await,
        "remove" => super::quotes::remove(interaction).await,
        "get" => super::quotes::get(interaction).await,
        "list" => super::quotes::list(interaction).await,
        _ => "Unknown Option!".to_string(),
    }
}

async fn handle_bigmoji_command(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    match cmd.name.as_str() {
        "add" => super::bigmoji::add(interaction).await,
        "remove" => super::bigmoji::remove(interaction).await,
        "get" => super::bigmoji::get(interaction).await,
        "list" => format!("http://{}{}/bigmoji", *DOMAIN, *PREFIX),
        _ => "Unknown Option!".to_string(),
    }
}

pub fn get_cmd(interaction: &Interaction) -> &CommandDataOption {
    if let Interaction::ApplicationCommand(inter) = interaction {
        inter.data.options.first().unwrap()
    } else {
        unreachable!()
    }
}
