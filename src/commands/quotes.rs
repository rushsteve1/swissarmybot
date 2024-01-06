use anyhow::bail;
use anyhow::Context;
use chrono::NaiveDateTime;
use serenity::all::UserId;
use serenity::all::{CommandDataOptionValue, Context as Ctx, Interaction, Mentionable};
use tracing::instrument;

use crate::helpers::get_cfg;
use crate::helpers::{get_cmd, get_db, get_inter};

#[derive(sqlx::FromRow)]
pub struct Quote {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub author_id: i64,
    pub author_name: String,
    pub text: String,
    pub inserted_at: NaiveDateTime,
}

#[instrument]
pub async fn add(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
    let db = get_db(ctx.clone()).await?;
    let cmd = get_cmd(interaction)?;

    let CommandDataOptionValue::SubCommand(cmds) = cmd.value.clone() else {
        bail!("was not subcommand");
    };

    let Some(user_id) = cmds.get(0).and_then(|u| u.value.as_user_id()) else {
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

    let id = user_id.to_string();
    let name = &user.name;
    let author_id = author.user.id.to_string();
    let author_name = author.user.name.to_string();

    sqlx::query!("INSERT INTO quotes (text, user_id, user_name, author_id, author_name) VALUES (?, ?, ?, ?, ?);",
                        text,
                        id, name, author_id, author_name)
                        .execute(&db)
                        .await
                        .with_context(|| "error inserting quote")?;

    Ok(format!("Quote added for {}\n>>> {}", user, text))
}

pub async fn remove(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
    let db = get_db(ctx).await?;
    let cmd = get_cmd(interaction)?;

    let CommandDataOptionValue::SubCommand(cmds) = cmd.value.clone() else {
        bail!("was not subcommand");
    };

    let id = cmds
        .get(0)
        .and_then(|c| c.value.as_i64())
        .ok_or(anyhow::anyhow!("quote get id"))?;

    let row = sqlx::query_scalar!("DELETE FROM quotes WHERE id = ? RETURNING user_id;", id)
        .fetch_optional(&db)
        .await
        .with_context(|| "error deleting quote")?;

    if let Some(user_id) = row {
        let user_id = serenity::model::id::UserId::new(user_id as u64);

        Ok(format!("Quote {} removed by {}", id, user_id.mention()))
    } else {
        Ok(format!("Quote {} does not exist", id))
    }
}

#[instrument]
pub async fn get(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
    let db = get_db(ctx).await?;
    let cmd = get_cmd(interaction)?;

    let CommandDataOptionValue::SubCommand(cmds) = cmd.value.clone() else {
        bail!("was not subcommand");
    };

    let id = cmds
        .get(0)
        .and_then(|c| c.value.as_i64())
        .ok_or(anyhow::anyhow!("quote get id"))?;

    let quote: Option<Quote> = sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = ?;", id)
        .fetch_optional(&db)
        .await
        .with_context(|| "error getting quote")?;

    // TODO embed reponse
    if let Some(quote) = quote {
        Ok(format!(
            "Quote {} by {}\n>>> {}",
            id,
            UserId::new(quote.user_id as u64).mention(),
            quote.text
        ))
    } else {
        Ok(format!("Quote {} does not exist", id))
    }
}

#[instrument]
pub async fn list(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
    let cfg = get_cfg(ctx).await?;
    let cmd = get_cmd(interaction)?;

    let CommandDataOptionValue::SubCommand(cmds) = cmd.value.clone() else {
        bail!("was not subcommand");
    };

    if let Some(user_id) = cmds.get(0).and_then(|u| u.value.as_user_id()) {
        Ok(format!("http://{}/quotes?user={}", cfg.addr, user_id))
    } else {
        Ok(format!("http://{}/quotes", cfg.addr))
    }
}
