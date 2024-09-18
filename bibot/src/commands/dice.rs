use crate::{Context, Error};
use anyhow::Result;
use rand::Rng;

#[poise::command(slash_command)]
pub async fn dice(ctx: Context<'_>) -> Result<(), Error> {
    let num = rand::thread_rng().gen_range(1..21);
    ctx.say(format!("Your number is: {num}")).await?;
    Ok(())
}
// Rolls a D20
