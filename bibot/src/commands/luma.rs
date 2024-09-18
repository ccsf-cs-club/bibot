use crate::{Context, Error};
use anyhow::Result;
use chrono::prelude::*;
use serde::Deserialize;

use chrono::TimeZone;
use chrono_tz::US::Pacific;

#[derive(Debug, Deserialize)]
struct GeoAddressInfo {
    city_state: String,
}

#[derive(Debug, Deserialize)]
struct Event {
    name: String,
    url: String,
    geo_address_info: GeoAddressInfo,
}

#[derive(Debug, Deserialize)]
struct Calendar {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Entry {
    event: Event,
    calendar: Calendar,
    start_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LumaEvents {
    entries: Vec<Entry>,
    has_more: bool,
    next_cursor: String,
}

#[poise::command(slash_command)]
pub async fn luma(ctx: Context<'_>) -> Result<(), Error> {
    // Ping the Luma public API for events
    let body = reqwest::get("https://api.lu.ma/discover/get-paginated-events?discover_place_api_id=discplace-BDj7GNbGlsF7Cka&pagination_limit=50")
        .await?
        .text()
        .await?;
    let events: LumaEvents = serde_json::from_str(&body)?;
    let mut message = String::from("Luma in SF: \n");
    let mut weekday: Weekday = Weekday::Sun;
    for entry in events.entries.into_iter() {
        let event = &entry.event;
        // let calendar = &entry.calendar;
        if event.geo_address_info.city_state == "San Francisco, California" {
            let dt = DateTime::parse_from_rfc3339(&entry.start_at.unwrap()).unwrap();
            let pst = dt.with_timezone(&Pacific);
            if pst.weekday() != weekday {
                weekday = pst.weekday();
                message += format!("{} ({}/{}):\n", weekday, pst.month(), pst.day()).as_str();
            }
            message += format!(
                "\t{}: [{}](https://lu.ma/{})\n",
                pst.format("%l%p"),
                &event.name,
                &event.url
            )
            .as_str();
        }
    }
    let mut chat_msg = String::from("");
    let mut wc = 0;
    for line in message.split("\n") {
        chat_msg.push_str(line);
        chat_msg.push('\n');
        wc += line.len();
        if wc >= 1500 {
            ctx.say(&chat_msg).await?;
            chat_msg = String::from("");
            wc = 0;
        }
    }
    // ctx.say(&chat_msg).await?;

    Ok(())
}
