use poise::serenity_prelude as serenity;

use crate::{Context, Error};

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

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

/// Reloads the slash commands in the guild
#[poise::command(slash_command, prefix_command)]
pub async fn reload_slash(ctx: Context<'_>) -> Result<(), Error> {
    match ctx.guild_id() {
        None => {
            ctx.say("Command only possible in Guild").await?;
        }
        Some(guild) => {
            poise::builtins::register_in_guild(ctx, &ctx.framework().options().commands, guild)
                .await?;
        }
    };
    Ok(())
}
