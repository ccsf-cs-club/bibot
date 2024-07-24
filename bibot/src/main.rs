use anyhow::Context as _;
use anyhow::Result;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

mod commands;

struct Data {
    notion_api_key: String,
    accuweather_api_key: String,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with a greeting
#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(", CCSF CS Club!").await?;
    Ok(())
}

// Makes it partytime for 1 hour
#[poise::command(slash_command)]
async fn partytime(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("/create title:dev_event datetime:in 5 seconds description:test dev event duration:15 seconds channel:#bot-spam").await?;
    Ok(())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;
    let notion_token = secret_store
        .get("NOTION_API_KEY")
        .context("'NOTION_API_KEY' was not found");
    let accuweather_api_key = secret_store
        .get("ACCUWEATHER_API_KEY")
        .context("'ACCUWEATHER_API_KEY' was not found");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                hello(),
                commands::dice::dice(),
                commands::iss::iss(),
                commands::luma::luma(),
                commands::notion::notion(),
                commands::version::version(),
                commands::weather::weather(),
                commands::linkedin::linkedin(),
                partytime(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    notion_api_key: notion_token.unwrap(),
                    accuweather_api_key: accuweather_api_key.unwrap(),
                })
            })
        })
        .build();

    let client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;
    Ok(client.into())
}
