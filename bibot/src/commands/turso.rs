use crate::{Context, Error};
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct Report {
    title: String,
    summary: String,
    news_site: String,
    url: String,
}

#[derive(Deserialize)]
struct Reports {
    results: Vec<Report>,
}

#[poise::command(slash_command)]
pub async fn iss(ctx: Context<'_>) -> Result<(), Error> {
    use libsql::Builder;

    let mut db = Builder::new_remote_replica("local.db", &url, &token)
        .build()
        .await
        .unwrap();
    let conn = db.connect().unwrap();

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
