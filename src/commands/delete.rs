use std::num::NonZeroU64;

use crate::models::{DBChannel, DBMate, DBMessage};
use crate::utils;

use super::autocomplete::mate as mate_autocomplete;
use super::CommandContext;
use anyhow::{Context, Result};
use mongodb::bson::doc;
use poise::serenity_prelude::{CacheHttp, MessageId};

#[poise::command(slash_command, subcommands("mate", "message"))]
pub async fn delete(_ctx: CommandContext<'_>) -> Result<()> {
    unreachable!()
}

/// Delete a mate
#[poise::command(slash_command, ephemeral)]
pub async fn mate(
    ctx: CommandContext<'_>,
    #[description = "name of the mate to delete"]
    #[autocomplete = "mate_autocomplete"]
    name: String,
) -> Result<()> {
    let database = &ctx.data().database;

    let mates_collection = database.collection::<DBMate>("mates");

    utils::get_mate(&mates_collection, ctx.author().id, name.clone())
        .await
        .context("Failed to find mate; do they actually exist?")?;

    utils::delete_mate(&mates_collection, ctx.author().id, name.clone()).await?;

    ctx.say("Successfully deleted mate! o7 :headstone:").await?;
    Ok(())
}

#[poise::command(slash_command, ephemeral)]
pub async fn message(
    ctx: CommandContext<'_>,
    #[description = "the raw ID of the message to delete"] message_id: Option<u64>,
    #[description = "a link to the message to delete"] message_link: Option<String>,
) -> Result<()> {
    let database = &ctx.data().database;
    let channels_collection = database.collection::<DBChannel>("channels");
    let messages_collection = database.collection::<DBMessage>("messages");

    let message_to_delete_id;
    if let Some(message_id) = message_id {
        message_to_delete_id = MessageId(NonZeroU64::new(message_id).unwrap())
    } else if let Some(message_link) = message_link {
        message_to_delete_id = utils::message_link_to_id(message_link)?
    } else {
        let message = utils::get_most_recent_message(&messages_collection, ctx.author().id).await?;
        message_to_delete_id = MessageId(NonZeroU64::new(message.message_id).unwrap())
    }

    let (webhook, thread_id) =
        utils::get_webhook_or_create(ctx.http(), &channels_collection, ctx.channel_id()).await?;

    let dbmessage = utils::get_message(
        &messages_collection,
        Some(ctx.author().id),
        message_to_delete_id,
    )
    .await;

    if let Ok(_) = dbmessage {
        webhook
            .delete_message(ctx.http(), thread_id, message_to_delete_id)
            .await?;

        ctx.say("Deleted message! o7 :headstone:").await?;
        Ok(())
    } else {
        // weird looking but just propagates the error generated by get_message to the user
        dbmessage?;
        Ok(())
    }
}
