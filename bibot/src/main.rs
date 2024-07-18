extern crate ical;
use ical::parser::{ical::component::IcalCalendar, Component};

use anyhow::Context as _;
use anyhow::Result;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use rand::Rng;
use rusticnotion::ids::BlockId;
use rusticnotion::ids::DatabaseId;
use rusticnotion::models::block::Block;
use rusticnotion::models::search::DatabaseQuery;
use rusticnotion::models::Database;
use rusticnotion::NotionApi;
use serde::{Deserialize, Serialize};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use std::cmp;
use std::str::FromStr;

struct Data {
    notion_api_key: String,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with a greeting
#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(", CCSF CS Club!").await?;
    Ok(())
}

/// Rolls a D6
#[poise::command(slash_command)]
async fn dice(ctx: Context<'_>) -> Result<(), Error> {
    let num = rand::thread_rng().gen_range(1..7);
    ctx.say(format!("You number is: {}", num)).await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Report {
    title: String,
    summary: String,
    news_site: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Reports {
    results: Vec<Report>,
}

/// Reports on the ISS' latest shenanigans
#[poise::command(slash_command)]
async fn iss(ctx: Context<'_>) -> Result<(), Error> {
    let body = reqwest::get("https://api.spaceflightnewsapi.net/v4/reports/?format=json&limit=1")
        .await?
        .text()
        .await?;
    let r: Reports = serde_json::from_str(&body)?;
    ctx.say(format!(
        "{}: {}\n{}\n\n[Read More]({})",
        r.results[0].news_site, r.results[0].title, r.results[0].summary, r.results[0].url
    ))
    .await?;
    Ok(())
}

fn format_date(dt: String) -> String {
    format!("{}/{}:\n", dt.get(4..6).unwrap(), dt.get(6..8).unwrap())
}

/// Reports on events in SF using Luma
#[poise::command(slash_command)]
async fn luma(ctx: Context<'_>, offset: usize, qty: usize) -> Result<(), Error> {
    let body =
        reqwest::get("https://api.lu.ma/ics/get?entity=discover&id=discplace-BDj7GNbGlsF7Cka")
            .await?
            .text()
            .await?;

    let reader = ical::IcalParser::new(body.as_bytes());
    let mut msg = String::with_capacity(2000);
    for line in reader {
        if let Ok(cal) = line {
            // Title
            if offset > cal.events.len() {
                ctx.say("No events").await?;
                return Ok(());
            }
            let end = cmp::min(offset + qty, cal.events.len());
            let title = cal
                .get_property("X-WR-CALNAME")
                .unwrap()
                .value
                .clone()
                .unwrap();
            msg.push_str(format!("{} - {}-{}\n", title, offset, end).as_str());

            // Events
            let mut dt = String::new();
            for event in &cal.events[offset..end] {
                let mut summary = event
                    .get_property("SUMMARY")
                    .unwrap()
                    .value
                    .clone()
                    .unwrap();

                // Sometimes the summary includes an @, use what comes before it
                summary = summary.split("@").take(1).collect();
                summary = summary.split(" - ").take(1).collect();
                summary = summary.split(" | ").take(1).collect();

                let time = event
                    .get_property("DTSTART")
                    .unwrap()
                    .value
                    .clone()
                    .unwrap();

                // If the date is a new one, display it
                if dt != format_date(time.clone()) {
                    dt = format_date(time.clone());
                    msg.push_str(dt.as_str());
                }

                let mut location = event
                    .get_property("LOCATION")
                    .unwrap()
                    .value
                    .clone()
                    .unwrap();
                // Similarly, don't need anything after ', San Francisco,...'
                if location.contains(", San Francisco") {
                    location = location.split(", San Francisco").take(1).collect();
                }
                location = location.split(", ").take(1).collect();

                if location.contains("http") {
                    msg.push_str(format!("- [{}]({})\n", summary.trim(), location.trim()).as_str());
                } else {
                    msg.push_str(format!("- {} @ {}\n", summary.trim(), location.trim()).as_str());
                }
            }
        }
    }

    ctx.say(msg).await?;
    Ok(())
}

// Lists upcoming events for the week
#[poise::command(slash_command)]
async fn this_week(ctx: Context<'_>) -> Result<(), Error> {
    let client: NotionApi = NotionApi::new(ctx.data().notion_api_key.clone())?;
    let db_id: DatabaseId = DatabaseId::from_str("bc689835f6904f83871cb67efe6b7d1e")?;
    let db: Database = client.get_database(db_id).await?;
    let query: DatabaseQuery = DatabaseQuery {
        sorts: None,
        filter: None,
        paging: None,
    };
    let response = client.query_database(db, query).await?;
    let page = response.results.get(0).expect("No events");
    let mut body = String::from("");
    // Title
    body.push_str(format!("**{}**\n", page.title().ok_or("Untitled")?).as_str());

    // Body Text
    let children = client
        .get_block_children(BlockId::from(page.id.clone()))
        .await?;
    for child in children.results {
        if let Block::Paragraph { paragraph, .. } = child {
            for t in paragraph.rich_text {
                body.push_str(t.plain_text())
            }
        }
    }

    ctx.say(format!(
        "Here are the upcoming events for the week in our calendar:\n{}",
        body
    ))
    .await?;
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

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![hello(), dice(), iss(), luma(), this_week(), partytime()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    notion_api_key: notion_token.expect("Error finding Notion API Key"),
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
