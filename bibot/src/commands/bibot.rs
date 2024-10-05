use crate::{Context, Error};
use anyhow::Result;

#[poise::command(slash_command)]
pub async fn bibot(ctx: Context<'_>) -> Result<(), Error> {
    // Diagnostic Command
    ctx.say("Foo").await?;

    Ok(())
}
