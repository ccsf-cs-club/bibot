use crate::{Context, Error};
use anyhow::Result;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// Scans and post job listings
#[poise::command(slash_command)]
pub async fn linkedin(
    ctx: Context<'_>,
    keywords: String,
    pages: usize,
    offset: usize,
) -> Result<(), Error> {
    let url = "https://www.linkedin.com/jobs-guest/jobs/api/seeMoreJobPostings/search";
    let location = "San%20Francisco,%20California,%20United%20States";
    let mut msg = String::new();
    for i in offset..offset+pages {
        let query = format!("{}?keywords={}&location={}&start={}", url, keywords, location, i * 10);
        let data = reqwest::get(query).await?.text().await?;
        let fragment = Html::parse_fragment(data.as_str());
        let selector = Selector::parse(".base-search-card__info").unwrap();

        let title_selector = Selector::parse(".base-search-card__title").unwrap();
        let company_selector = Selector::parse(".base-search-card__subtitle > a").unwrap();

        for element in fragment.select(&selector) {
            msg.push_str(format!(
                "{} @ {}\n",
                element
                    .select(&title_selector)
                    .next()
                    .unwrap()
                    .inner_html()
                    .trim(),
                element
                    .select(&company_selector)
                    .next()
                    .unwrap()
                    .inner_html()
                    .trim()
            ).as_str());
        }
    }
    if msg == "" {
        msg = String::from("No jobs found.");
    }
    ctx.say(format!("{}", msg)).await?;
    Ok(())
}
