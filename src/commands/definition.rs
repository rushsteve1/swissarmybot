use serenity::client::Context;
use serenity::model::application::command::Command;
use serenity::model::application::command::*;
use serenity::model::id::GuildId;

pub async fn clear_definitions(ctx: &Context) {
    let commands = Command::get_global_application_commands(&ctx.http)
        .await
        .unwrap();

    for command in commands {
        Command::delete_global_application_command(&ctx.http, command.id)
            .await
            .unwrap();
    }
}

pub async fn clear_definitions_for_guild(ctx: &Context, guild_id: GuildId) {
    let commands = ctx
        .http
        .get_guild_application_commands(guild_id.into())
        .await
        .unwrap();

    for command in commands {
        ctx.http
            .delete_guild_application_command(guild_id.into(), command.id.into())
            .await
            .unwrap();
    }
}

/// Builds the definition of the slash command "interactions" and sends it to
/// Discord where it can will be displayed
pub async fn interactions_definition(ctx: Context) -> Vec<Command> {
    Command::set_global_application_commands(&ctx.http, |commands| {
        commands
            .create_application_command(|command| {
                command
                    .name("quote")
                    .description("Manage peoples' quotes")
                    .create_option(|option| {
                        option
                            .name("add")
                            .kind(CommandOptionType::SubCommand)
                            .description("Add a quote to the database")
                            .create_sub_option(|option| {
                                option
                                    .name("who")
                                    .required(true)
                                    .kind(CommandOptionType::User)
                                    .description("Who is this quote by?")
                            })
                            .create_sub_option(|option| {
                                option
                                    .name("text")
                                    .kind(CommandOptionType::String)
                                    .required(true)
                                    .description("What did they say?")
                            })
                    })
                    .create_option(|option| {
                        option
                            .name("remove")
                            .kind(CommandOptionType::SubCommand)
                            .description("Remove a quote from the database")
                            .create_sub_option(|option| {
                                option
                                    .name("number")
                                    .kind(CommandOptionType::Integer)
                                    .required(true)
                                    .description("What number quote should be removed?")
                            })
                    })
                    .create_option(|option| {
                        option
                            .name("get")
                            .kind(CommandOptionType::SubCommand)
                            .description("Get a quote from the database")
                            .create_sub_option(|option| {
                                option
                                    .name("number")
                                    .kind(CommandOptionType::Integer)
                                    .required(true)
                                    .description("What number quote should be gotten?")
                            })
                    })
                    .create_option(|option| {
                        option
                            .name("list")
                            .kind(CommandOptionType::SubCommand)
                            .description("List all the quotes by this user")
                            .create_sub_option(|option| {
                                option
                                    .name("who")
                                    .kind(CommandOptionType::User)
                                    .description("Who is this quote by?")
                            })
                    })
            })
            .create_application_command(|command| {
                command
                    .name("bigmoji")
                    .description("Manage BigMoji (big emoji)")
                    .create_option(|option| {
                        option
                            .name("add")
                            .kind(CommandOptionType::SubCommand)
                            .description("Add a BigMoji to the database")
                            .create_sub_option(|option| {
                                option
                                    .name("name")
                                    .kind(CommandOptionType::String)
                                    .required(true)
                                    .description("Name of the BigMoji (without colons)")
                            })
                            .create_sub_option(|option| {
                                option
                                    .name("text")
                                    .kind(CommandOptionType::String)
                                    .required(true)
                                    .description("What should it say? (links OK)")
                            })
                    })
                    .create_option(|option| {
                        option
                            .name("remove")
                            .kind(CommandOptionType::SubCommand)
                            .description("Remove a BigMoji from the database")
                            .create_sub_option(|option| {
                                option
                                    .name("name")
                                    .kind(CommandOptionType::String)
                                    .required(true)
                                    .description("Name of the BigMoji (without colons)")
                            })
                    })
                    .create_option(|option| {
                        option
                            .name("get")
                            .kind(CommandOptionType::SubCommand)
                            .description("Get a BigMoji from the database")
                            .create_sub_option(|option| {
                                option
                                    .name("name")
                                    .kind(CommandOptionType::String)
                                    .required(true)
                                    .description("Name of the BigMoji (without colons)")
                            })
                    })
                    .create_option(|option| {
                        option
                            .name("list")
                            .kind(CommandOptionType::SubCommand)
                            .description("List all the BigMoji")
                    })
            })
    })
    .await
    .expect("Error sending interaction data to Discord")
}
