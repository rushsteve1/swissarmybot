use serenity::all::{CommandDataOptionValue, Interaction, Mentionable};
use tracing::error;

use crate::shared::helpers::{get_cmd, get_inter};

pub async fn update(db: sqlx::SqlitePool, interaction: &Interaction) -> anyhow::Result<String> {
	let inter = get_inter(interaction)?;
	let cmd = get_cmd(interaction)?;

	let author = inter
		.member
		.as_ref()
		.ok_or(anyhow::anyhow!("interaction had no author"))?;
	let author_id = author.user.id;
	let author_name = author.user.name.to_string();

	let drink_type = cmd.name.as_str();
	let CommandDataOptionValue::SubCommand(subcmds) = cmd.value.clone() else {
		error!("command value was not a subcommand");
		return Err(anyhow::anyhow!("command value was not a subcommand"));
	};
	let drink_name = subcmds.first().and_then(|d| d.value.as_str());

	let (author_id, type_str) =
		crate::shared::drunks::update(db, author_id, author_name.as_str(), drink_type, drink_name)
			.await?;

	Ok(format!("{} had a {}", author_id.mention(), type_str))
}
