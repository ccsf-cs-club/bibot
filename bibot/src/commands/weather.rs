use crate::{Context, Error};
use anyhow::Result;

/// Post the weather this week:
#[poise::command(slash_command)]
pub async fn weather(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!("WIP")).await?;
    Ok(())
}
