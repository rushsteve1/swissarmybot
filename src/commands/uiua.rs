use std::time;

use poise::{command, serenity_prelude::Message};
use tracing::instrument;
use uiua::{Compiler, Uiua};

use crate::Ctx;

const TIMEOUT: time::Duration = time::Duration::from_secs(15);

/// Run a snippet of code in the Uiua programming language. 15 second limit.
#[command(slash_command)]
#[instrument]
pub async fn uiua(
	ctx: Ctx<'_>,
	#[description = "The Uiua code to run"] code: String,
) -> anyhow::Result<()> {
	run_uiua(ctx, &code).await
}

#[poise::command(context_menu_command = "Run Uiua")]
#[instrument]
pub async fn context_menu(
	ctx: Ctx<'_>,
	#[description = "The message to run as Uiua"] msg: Message,
) -> anyhow::Result<()> {
	let code = msg.content.trim_matches('`').trim();

	run_uiua(ctx, code).await
}

#[instrument]
async fn run_uiua(ctx: Ctx<'_>, code: &str) -> anyhow::Result<()> {
	// TODO move the compiler to the context
	let mut comp = Compiler::new();

	if let Err(e) = comp.load_str(code) {
		ctx.say(format!("Uiua compiler error\n>>> {e}")).await?;
		return Ok(());
	}
	let asm = comp.finish();

	// TODO is this safe? Do I care?
	let mut inter = Uiua::with_safe_sys();
	let res = tokio::time::timeout(TIMEOUT, async { inter.run_asm(asm) }).await?;

	match res {
		Ok(_) => {
			ctx.say(format!(
				"Uiua program completed with stack:\n```\n{}\n```",
				format_stack(inter.stack())
			))
			.await?;
		}
		Err(e) => {
			ctx.say(format!("Uiua program errored:\n>>> {e}")).await?;
		}
	}

	Ok(())
}

fn format_stack(stack: &[uiua::Value]) -> String {
	stack.iter().map(|v| v.to_string() + "\n").collect()
}
