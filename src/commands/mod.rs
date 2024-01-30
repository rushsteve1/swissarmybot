pub mod drunks;
pub mod events;
pub mod quotes;
pub mod uiua;

#[poise::command(slash_command, owners_only)]
#[tracing::instrument]
pub async fn register(ctx: crate::Ctx<'_>) -> anyhow::Result<()> {
	poise::builtins::register_application_commands_buttons(ctx).await?;
	Ok(())
}
