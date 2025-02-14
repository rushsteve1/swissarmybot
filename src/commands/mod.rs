pub mod events;
pub mod quotes;
pub mod spill;

#[tracing::instrument]
#[poise::command(slash_command, owners_only)]
pub async fn register(ctx: crate::Ctx<'_>) -> anyhow::Result<()> {
	poise::builtins::register_application_commands_buttons(ctx).await?;

	Ok(())
}
