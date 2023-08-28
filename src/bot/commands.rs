use core::time;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::SystemTime;

use futures::{Stream, StreamExt};
use log::{error, info, warn};
use poise::serenity_prelude as serenity;
use serenity::{futures, AttachmentType};

use crate::bot::{Context, Error};
use crate::{download_watcher, xml};

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
    let reply_message = ctx.say(response).await?;
    reply_message
        .edit(ctx, |builder| {
            builder.content(match now.elapsed() {
                Ok(elapsed) => {
                    format!("Pong: {} ms", elapsed.as_millis())
                }
                Err(_) => "Pong: could not calculate time difference".to_owned(),
            })
        })
        .await?;
    Ok(())
}

/// Reloads the slash commands in the guild
#[poise::command(slash_command, prefix_command)]
pub async fn reload_slash(ctx: Context<'_>) -> Result<(), Error> {
    match ctx.guild_id() {
        None => ctx.say("Command only possible in Guild"),
        Some(guild) => {
            poise::builtins::register_in_guild(ctx, &ctx.framework().options().commands, guild)
                .await?;
            ctx.say("Reloaded slash commands")
        }
    }
    .await?;
    Ok(())
}

/// Stops the bot
#[poise::command(slash_command, prefix_command, aliases("shutdown"))]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Stopping bot").await?;
    if let Some(tx) = &ctx.data().tx {
        if let Err(why) = tx.try_send(download_watcher::SIGNAL_STOP) {
            error!("Could not stop Thread {:?}", why);
        }
    }
    sleep(time::Duration::from_secs(1));
    ctx.framework()
        .shard_manager
        .lock()
        .await
        .shutdown_all()
        .await;
    Ok(())
}

/// Reloads all directories
#[poise::command(slash_command, prefix_command)]
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    info!("Reloading all Directories");
    if let Some(tx) = &ctx.data().tx {
        match tx.try_send(download_watcher::SIGNAL_RELOAD) {
            Ok(_) => ctx.say("Reloaded all Directories"),
            Err(why) => {
                error!("Could not reload Directories {:?}", why);
                ctx.say("Couldn't reload Directories, try again later.")
            }
        }
        .await?;
    }
    Ok(())
}

/// Parent Map Command
#[poise::command(slash_command, subcommands("all", "new"))]
pub async fn map(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Will show all Mappings
#[poise::command(slash_command, prefix_command)]
pub async fn all(ctx: Context<'_>) -> Result<(), Error> {
    let mut mappings: HashMap<String, Vec<&str>> = HashMap::new();
    let reverse_mapping = xml::get_mappings();
    reverse_mapping.iter().for_each(|(to_replace, og)| {
        if let Some(list) = mappings.get_mut(og) {
            list.push(to_replace);
        } else {
            mappings.insert(og.to_string(), vec![to_replace]);
        }
    });
    let output = mappings
        .iter()
        .map(|(og, list)| {
            format!(
                "{} : {}\n",
                og,
                (*list).join(format!("\n{}", " ".repeat(og.len() + 3)).as_str())
            )
        })
        .reduce(|x, x1| format!("{}{}", x, x1))
        .unwrap_or("No mappings".to_string());
    ctx.send(|builder| {
        builder
            .content("Here are all the Mappings")
            .attachment(AttachmentType::Bytes {
                data: Cow::from(output.as_bytes()),
                filename: "mappings.txt".to_string(),
            })
    })
    .await?;
    Ok(())
}

/// autocomplete the known files that don't have mappings
async fn autocomplete_alt<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let missing_mappings: Vec<String>;
    if let Some(shared_data) = &_ctx.data().shared_thread_infos {
        missing_mappings = shared_data.lock().unwrap().missing_mappings.clone();
    } else {
        missing_mappings = Vec::new();
    }
    futures::stream::iter(missing_mappings)
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(|name| name.to_string())
}

/// autocomplete the known folders
async fn autocomplete_og<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let directories: HashMap<String, PathBuf>;
    if let Some(shared_data) = &_ctx.data().shared_thread_infos {
        directories = shared_data.lock().unwrap().og_directories.clone();
    } else {
        directories = HashMap::new();
    }
    futures::stream::iter(directories.into_iter())
        .filter(move |(name, _)| futures::future::ready(name.starts_with(partial)))
        .map(|(name, _)| name.to_string())
}

/// Will add a new Mapping to the Bot
#[poise::command(slash_command)]
pub async fn new(
    ctx: Context<'_>,
    #[description = "alternative name"]
    #[autocomplete = "autocomplete_alt"]
    mut alt: String,
    #[description = "series name on server"]
    #[autocomplete = "autocomplete_og"]
    mut og: String,
) -> Result<(), Error> {
    alt = alt.to_lowercase();
    og = og.to_lowercase();
    if let Some(shared_data) = &ctx.data().shared_thread_infos {
        if shared_data
            .lock()
            .unwrap()
            .og_directories
            .contains_key(og.as_str())
        {
            info!("Adding new Mapping");
            let message = ctx.say("Done".to_string());
            {
                shared_data
                    .lock()
                    .unwrap()
                    .missing_mappings
                    .retain(|x| x.deref() != alt);
            }
            xml::add_mappings(alt, og);
            if let Some(tx) = &ctx.data().tx {
                tx.send(download_watcher::SIGNAL_NEW_MAPPING)?;
            }
            message.await?;
        } else {
            warn!("Mapping could not be added");
            ctx.say(format!("Don't know `{}` please try again.", og))
                .await?;
        }
    } else {
        warn!("Mapping Thread not started");
        ctx.say("Mapping Thread not started".to_string()).await?;
    }

    Ok(())
}
