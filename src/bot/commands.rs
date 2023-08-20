use core::time;
use std::process::exit;
use std::thread::sleep;
use std::time::SystemTime;

use poise::{say_reply, serenity_prelude as serenity};

use crate::bot::{Context, Error};

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    ).await?;
    Ok(())
}

/// ping command
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let response = "Pong!";
    let now = SystemTime::now();
    let reply_message = say_reply(ctx, response).await?;
    reply_message.edit(ctx, |builder| {
        builder.content(match now.elapsed() {
            Ok(elapsed) => {format!("Pong: {} ms", elapsed.as_millis())},
            Err(_) => "Pong: could not calculate time difference".to_owned(),
        })
    }).await?;
    Ok(())
}

/// Reloads the slash commands in the guild
#[poise::command(slash_command, prefix_command)]
pub async fn reload_slash(ctx: Context<'_>) -> Result<(), Error> {
    match ctx.guild_id() {
        None => say_reply(ctx, "Command only possible in Guild"),
        Some(guild) => {
            poise::builtins::register_in_guild(ctx, &ctx.framework().options().commands, guild)
                .await?;
            say_reply(ctx, "Reloaded slash commands")
        }
    }
    .await?;
    Ok(())
}

/// Stops the bot
#[poise::command(slash_command, prefix_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    say_reply(ctx, "Stopping bot").await?;
    sleep(time::Duration::from_secs(1));
    exit(0);
}

/// Reloads all directories
#[poise::command(slash_command, prefix_command)]
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    // TODO: Implement
    Ok(())
}
