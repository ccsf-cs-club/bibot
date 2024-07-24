use crate::{Context, Error};
use anyhow::Result;

/// Prints this bot's version
#[poise::command(slash_command)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    ctx.say(format!("Bibot v{}", VERSION)).await?;
    Ok(())
}
