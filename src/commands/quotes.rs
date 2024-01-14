use anyhow::bail;
use serenity::all::{CommandDataOptionValue, Context as Ctx, Interaction, Mentionable};

use crate::shared::helpers::{get_cfg, get_cmd, get_db, get_inter};
use crate::shared::quotes;

pub async fn add(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
	let db = get_db(ctx.clone()).await?;
	let cmd = get_cmd(interaction)?;

	let CommandDataOptionValue::SubCommand(cmds) = cmd.value.clone() else {
		bail!("was not subcommand");
	};

	let Some(user_id) = cmds.first().and_then(|u| u.value.as_user_id()) else {
		bail!("no user id");
	};
	let user = user_id.to_user(ctx).await?;
	let Some(text) = cmds.get(1).and_then(|t| t.value.as_str()) else {
		bail!("no quote text")
	};

	let inter = get_inter(interaction)?;
	let Some(author) = inter.member.as_ref() else {
		bail!("no quote author")
	};

	let user_name = &user.name;
	let author_id = author.user.id;
	let author_name = author.user.name.clone();

	quotes::add(
		db,
		user_id,
		user_name,
		author_id,
		author_name.as_str(),
		text,
	)
	.await?;

	Ok(format!(
		"Quote added for {}\n>>> {}",
		user_id.mention(),
		text
	))
}

pub async fn remove(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
	let db = get_db(ctx).await?;
	let cmd = get_cmd(interaction)?;

	let CommandDataOptionValue::SubCommand(cmds) = cmd.value.clone() else {
		bail!("was not subcommand");
	};

	let id = cmds
		.first()
		.and_then(|c| c.value.as_i64())
		.ok_or(anyhow::anyhow!("quote get id"))?;

	quotes::remove(db, id).await
}

pub async fn get(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
	let db = get_db(ctx).await?;
	let cmd = get_cmd(interaction)?;

	let CommandDataOptionValue::SubCommand(cmds) = cmd.value.clone() else {
		bail!("was not subcommand");
	};

	let id = cmds
		.first()
		.and_then(|c| c.value.as_i64())
		.ok_or(anyhow::anyhow!("quote get id"))?;

	quotes::get_one(db, id).await
}

pub async fn list(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
	let cfg = get_cfg(ctx).await?;
	let cmd = get_cmd(interaction)?;

	let CommandDataOptionValue::SubCommand(cmds) = cmd.value.clone() else {
		bail!("was not subcommand");
	};

	let user_id = cmds.first().and_then(|u| u.value.as_user_id());

	Ok(quotes::list_url(cfg, user_id))
}
