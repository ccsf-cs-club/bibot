use crate::{Context, Error};
use anyhow::Result;
use chrono::prelude::*;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::fmt::Debug;

use langchain_rust::{
    chain::Chain,
    language_models::llm::LLM,
    llm::openai::{OpenAI, OpenAIConfig, OpenAIModel},
};

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

// #[poise::command(slash_command)]
pub async fn luma(ctx: Context<'_>) -> Result<(), Error> {
    // Ping the Luma public API for events
    let openai_key = &ctx.data().openai_key;
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
            let url = format!("https://lu.ma/{}", &event.url);

            let html = reqwest::get(url).await?.text().await?;
            let document = Html::parse_document(&html);
            let selector = Selector::parse(".event-about-card").unwrap();

            let event_info = document.select(&selector).next().unwrap();
            let text = event_info.text().collect::<Vec<_>>().join("\n");
            let open_ai = OpenAI::default()
                .with_config(OpenAIConfig::default().with_api_key(openai_key))
                .with_model(OpenAIModel::Gpt4oMini.to_string());

            let prompt = format!("You are a starving college student named Smallnumbers. Your mission in life is to help identify free food at tech events. You will be given event descriptions, and your job is to determine whether or not food is likely to be served. If there is food, mention what type it'll be. Give the reasoning in a 'comment' blurb inside the object you'll return.
        --------
            Event: 
            Join us for the Dreamforce After Party! Dive into the world of Generative AI with an exclusive gathering that combines delicious sushi, themed beverages, and stimulating conversations on the transformative impact of AI technologies on corporations. This event will feature engaging discussions with industry leaders from AWS and Observea providing unique insights into the latest tools and technologies shaping the future of AI.

            Agenda

            5:00 Doors Open & Networking

            6:00 Welcoming Remarks

            6:15 Short Demo/Panel Discussions

            Panel Conversation: How GenAI will change how we work

            Panelists: Shannon Brownlee (Valence Vibrations), Vamsi Pandari/Chris Leggett (Observea), Shaun VanWeelden (ex ScaleAI, OpenAI)

            Lightning Presenters:

            Observea (Vamsi Pandari)

            Open Babylon (Yurii Filipchuk)

            SylphAI & AdalFlow Demo (Li Yin)

            6:30 Dinner & Drinks

            9:00 Event Ends

            10:00 Doors Close

            Space

            Step into the enchanting Roka Akor in San Francisco for the Dreamforce Gen AI After-Party, hosted by Observea and AWS. This exclusive venue combines modern sophistication with a warm, inviting atmosphere, making it a prime location for networking and insightful discussions on AI integration. Located in the historic Jackson Square, Roka Akor offers a stunning backdrop with its contemporary, chef-driven menu featuring prime steak, sushi, and seafood—all prepared on a signature robata grill.

            Throughout the evening, enjoy a selection of exquisite dinner and bite-sized treats provided by your hosts, Observea and AWS, ensuring a delightful culinary experience.

            This is your chance to network with the best in the industry and discuss future AI innovations. Spots are limited and exclusive to approved attendees—no +1s. If you were not pre-approved, unfortunately, you will not be admitted to the event.
        --------
            Answer: {{'has_food': true, 'food_type': 'steak, sushi, seafood', 'contributions_required': false, 'name': 'Dreamforce Gen AI After-Party', 'comment': 'Sushi, beverages, and steaks/seafood at Roka Akor in San Francisco mentioned explicitly.'}}
        --------
            Event:

            {}
        --------
            Answer: {{'has_free_food': ", text);
            let resp = open_ai.invoke(prompt.as_str());
            message += format!("{{'has_food': {}", resp.await.unwrap()).as_str();
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
    ctx.say(&chat_msg).await?;

    Ok(())
}

async fn food_report(url: &str, openai_api_key: &str) -> Result<String> {
    let html = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html);
    let selector = Selector::parse(".event-about-card").unwrap();

    let event_info = document.select(&selector).next().unwrap();
    let text = event_info.text().collect::<Vec<_>>().join("\n");
    let open_ai = OpenAI::default()
        .with_config(OpenAIConfig::default().with_api_key(openai_api_key))
        .with_model(OpenAIModel::Gpt4oMini.to_string());

    let prompt = format!("You are a starving college student named Smallnumbers. Your mission in life is to help identify free food at tech events. You will be given event descriptions, and your job is to determine whether or not food is likely to be served. If there is food, mention what type it'll be. Give the reasoning in a 'comment' blurb inside the object you'll return.
--------
    Event: 
    Join us for the Dreamforce After Party! Dive into the world of Generative AI with an exclusive gathering that combines delicious sushi, themed beverages, and stimulating conversations on the transformative impact of AI technologies on corporations. This event will feature engaging discussions with industry leaders from AWS and Observea providing unique insights into the latest tools and technologies shaping the future of AI.

    Agenda

    5:00 Doors Open & Networking

    6:00 Welcoming Remarks

    6:15 Short Demo/Panel Discussions

    Panel Conversation: How GenAI will change how we work

    Panelists: Shannon Brownlee (Valence Vibrations), Vamsi Pandari/Chris Leggett (Observea), Shaun VanWeelden (ex ScaleAI, OpenAI)

    Lightning Presenters:

    Observea (Vamsi Pandari)

    Open Babylon (Yurii Filipchuk)

    SylphAI & AdalFlow Demo (Li Yin)

    6:30 Dinner & Drinks

    9:00 Event Ends

    10:00 Doors Close

    Space

    Step into the enchanting Roka Akor in San Francisco for the Dreamforce Gen AI After-Party, hosted by Observea and AWS. This exclusive venue combines modern sophistication with a warm, inviting atmosphere, making it a prime location for networking and insightful discussions on AI integration. Located in the historic Jackson Square, Roka Akor offers a stunning backdrop with its contemporary, chef-driven menu featuring prime steak, sushi, and seafood—all prepared on a signature robata grill.

    Throughout the evening, enjoy a selection of exquisite dinner and bite-sized treats provided by your hosts, Observea and AWS, ensuring a delightful culinary experience.

    This is your chance to network with the best in the industry and discuss future AI innovations. Spots are limited and exclusive to approved attendees—no +1s. If you were not pre-approved, unfortunately, you will not be admitted to the event.
--------
    Answer: {{'has_food': true, 'food_type': 'steak, sushi, seafood', 'contributions_required': false, 'name': 'Dreamforce Gen AI After-Party', 'comment': 'Sushi, beverages, and steaks/seafood at Roka Akor in San Francisco mentioned explicitly.'}}
--------
    Event:

    {}
--------
    Answer: {{'has_free_food': ", text);
    let resp = open_ai.invoke(prompt.as_str()).await.unwrap();
    println!("{{'has_food': {}", resp);
    Ok(String::from(url))
}
