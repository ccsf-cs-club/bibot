use crate::{Context, Error};
use anyhow::Result;
use serde::{Deserialize, Serialize};

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
pub async fn iss(ctx: Context<'_>) -> Result<(), Error> {
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
