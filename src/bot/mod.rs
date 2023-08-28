use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::info;
use poise::serenity_prelude as serenity;
use serenity::CacheHttp;

use crate::download_watcher::ThreadInfos;
use crate::{download_watcher, xml};

mod commands;

/// User data, which is stored and accessible in all command invocations
pub struct Data {
    tx: Option<SyncSender<u8>>,
    shared_thread_infos: Option<Arc<Mutex<ThreadInfos>>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn entrypoint() {
    info!("Starting the bot");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::help(),
                commands::reload_slash(),
                commands::ping(),
                commands::stop(),
                commands::map(),
            ],
            allowed_mentions: Some({
                let mut f = serenity::CreateAllowedMentions::default();
                f.empty_parse()
                    .parse(serenity::ParseValue::Users)
                    .replied_user(true);
                f
            }),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
                ..Default::default()
            },
            pre_command: |ctx| {
                Box::pin(async move {
                    info!(
                        "Received Command from {} in channel {}: {}",
                        ctx.author().name,
                        ctx.channel_id()
                            .name(ctx.cache().unwrap())
                            .await
                            .unwrap_or_else(|| { "Unknown".to_string() }),
                        ctx.invocation_string()
                    );
                })
            },
            ..Default::default()
        })
        .token(xml::get_bot_token())
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", _ready.user.name);
                ctx.set_activity(serenity::Activity::watching("downloads"))
                    .await;
                match download_watcher::entrypoint(ctx) {
                    None => framework.shard_manager().lock().await.shutdown_all().await,
                    Some((tx, shared_thread_infos)) => {
                        return Ok(Data {
                            tx: Some(tx),
                            shared_thread_infos: Some(shared_thread_infos),
                        })
                    }
                }
                Ok(Data {
                    tx: None,
                    shared_thread_infos: None,
                })
            })
        });

    framework.run().await.expect("Err creating client");
}
