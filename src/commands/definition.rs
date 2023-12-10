use serenity::all::{
    Command, CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId,
};

pub async fn clear_definitions(ctx: &Context) {
    let commands = Command::get_global_commands(&ctx.http).await.unwrap();

    for command in commands {
        Command::delete_global_command(&ctx.http, command.id)
            .await
            .unwrap();
    }
}

pub async fn clear_definitions_for_guild(ctx: &Context, guild_id: GuildId) {
    let commands = ctx.http.get_guild_commands(guild_id).await.unwrap();

    for command in commands {
        ctx.http
            .delete_guild_command(guild_id, command.id)
            .await
            .unwrap();
    }
}

/// Builds the definition of the slash command "interactions" and sends it to
/// Discord where it can will be displayed
pub async fn interactions_definition(ctx: Context) -> Vec<Command> {
    let quote_cmd = CreateCommand::new("quote")
        .description("Manage peoples' quotes")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "add",
                "Add a quote to the database",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::User, "who", "Who is this quote by?")
                    .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "text", "What did they say?")
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "remove",
                "Remove a quote from the database",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "number",
                    "What number quote should be removed?",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "get",
                "Get a quote from the database",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "number",
                    "What number quote should be gotten?",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "list",
                "List all the quotes by this user",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::User, "who", "Who is this quote by?")
                    .required(true),
            ),
        );

    let bigmoji_cmd = CreateCommand::new("bigmoji")
        .description("Manage BigMoji (big emoji)")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "add",
                "Add a BigMoji to the database",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "name",
                    "Name of the BigMoji (without colons)",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "text",
                    "What should it say? (links OK)",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "remove",
                "Remove a BigMoji from the database",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "name",
                    "Name of the BigMoji (without colons)",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "get",
                "Get a BigMoji from the database",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "name",
                    "Name of the BigMoji (without colons)",
                )
                .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "List all the BigMoji",
        ));

    Command::set_global_commands(&ctx.http, vec![quote_cmd, bigmoji_cmd])
        .await
        .expect("Error sending interaction data to Discord")
}
