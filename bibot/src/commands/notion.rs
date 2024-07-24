use crate::{Context, Error};
use anyhow::Result;

use rusticnotion::ids::BlockId;
use rusticnotion::ids::DatabaseId;
use rusticnotion::models::block::Block;
use rusticnotion::models::search::DatabaseQuery;
use rusticnotion::models::Database;
use rusticnotion::NotionApi;
use std::str::FromStr;

// Lists upcoming events for the week
#[poise::command(slash_command)]
pub async fn notion(ctx: Context<'_>) -> Result<(), Error> {
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
