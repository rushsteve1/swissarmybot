use anyhow::bail;
use anyhow::Context;
use chrono::NaiveDateTime;
use serenity::all::{CommandDataOptionValue, Context as Ctx, Interaction, Mentionable, UserId};
use sqlx::SqlitePool;
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

    add_handler(
        db,
        user_id,
        user_name,
        author_id,
        author_name.as_str(),
        text,
    )
    .await
}

#[instrument]
async fn add_handler(
    db: SqlitePool,
    user_id: UserId,
    user_name: &str,
    author_id: UserId,
    author_name: &str,
    text: &str,
) -> anyhow::Result<String> {
    let user_id_s = user_id.to_string();
    let author_id_s = author_id.to_string();

    sqlx::query!("INSERT INTO quotes (text, user_id, user_name, author_id, author_name) VALUES (?, ?, ?, ?, ?);",
                        text,
                        user_id_s, user_name, author_id_s, author_name)
                        .execute(&db)
                        .await
                        .with_context(|| "error inserting quote")?;

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

    remove_handler(db, id).await
}

#[instrument]
async fn remove_handler(db: SqlitePool, id: i64) -> anyhow::Result<String> {
    let row = sqlx::query_scalar!("DELETE FROM quotes WHERE id = ? RETURNING user_id;", id)
        .fetch_optional(&db)
        .await
        .with_context(|| "error deleting quote")?;

    Ok(row
        .map(|i| UserId::new(i as u64))
        .map(|user_id| format!("Quote {} removed by {}", id, user_id.mention()))
        .unwrap_or(format!("Quote {} does not exist", id)))
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

    get_handler(db, id).await
}

#[instrument]
async fn get_handler(db: SqlitePool, id: i64) -> anyhow::Result<String> {
    let quote: Option<Quote> = sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = ?;", id)
        .fetch_optional(&db)
        .await
        .with_context(|| "error getting quote")?;

    Ok(quote
        .map(|q| {
            format!(
                "Quote {} by {}\n>>> {}",
                id,
                UserId::new(q.user_id as u64).mention(),
                q.text
            )
        })
        .unwrap_or(format!("Quote {} does not exist", id)))
}

pub async fn list(ctx: Ctx, interaction: &Interaction) -> anyhow::Result<String> {
    let cfg = get_cfg(ctx).await?;
    let cmd = get_cmd(interaction)?;

    let CommandDataOptionValue::SubCommand(cmds) = cmd.value.clone() else {
        bail!("was not subcommand");
    };

    let user_id = cmds.first().and_then(|u| u.value.as_user_id());

    Ok(list_handler(cfg, user_id))
}

#[instrument]
fn list_handler(cfg: crate::Config, user_id: Option<UserId>) -> String {
    user_id
        .map(|u| format!("http://{}/quotes?user={}", cfg.addr, u))
        .unwrap_or(format!("http://{}/quotes", cfg.addr))
}
