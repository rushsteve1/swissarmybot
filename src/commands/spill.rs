use poise::serenity_prelude::Member;
use tracing::instrument;

use crate::Ctx;

const TYPST_DOC: &str = include_str!("../../spill.typ");

#[instrument]
#[poise::command(slash_command)]
async fn spill(
	ctx: Ctx<'_>,
	#[description = "Who spilled?"] user: Member,
	#[description = "What did they spill?"] liquid: String,
) -> anyhow::Result<()> {
	let reporter = ctx.author();

	typst::compile(TYPST_DOC).unwrap();

	unimplemented!()
}
