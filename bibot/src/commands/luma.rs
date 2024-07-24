extern crate ical;
use crate::{Context, Error};
use anyhow::Result;

#[allow(unused_imports)]
use ical::parser::{ical::component::IcalCalendar, Component};

use std::cmp;

fn format_date(dt: String) -> String {
    format!("{}/{}:\n", dt.get(4..6).unwrap(), dt.get(6..8).unwrap())
}

/// Reports on events in SF using Luma
#[poise::command(slash_command)]
pub async fn luma(ctx: Context<'_>, offset: usize, qty: usize) -> Result<(), Error> {
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

// Makes it partytime for 1 hour
#[poise::command(slash_command)]
async fn partytime(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("/create title:dev_event datetime:in 5 seconds description:test dev event duration:15 seconds channel:#bot-spam").await?;
    Ok(())
}
