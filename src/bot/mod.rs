use std::time::Duration;

use log::info;
use poise::{async_trait, serenity_prelude as serenity};
use poise::serenity_prelude::CacheHttp;
use crate::xml;

mod commands;

/// User data, which is stored and accessible in all command invocations
pub struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Handler;

#[async_trait]
impl serenity::EventHandler for Handler {
    async fn message(&self, context: serenity::Context, msg: serenity::Message) {
        if msg.content.starts_with("!") {
            info!("Received message from {} in channel {}: {}",
                        msg.author.name,
                        msg.channel_id.name(context.cache().unwrap()).await.unwrap_or_else(|| {"Unknown".to_string()}),
                        msg.content.to_string());
        }
    }

    async fn ready(&self, _: serenity::Context, ready: serenity::Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

pub async fn entrypoint() {
    info!("Starting the bot");
    let framework = poise::Framework::builder().options(poise::FrameworkOptions {
        commands: vec![
            commands::help(),
            commands::reload_slash(),
            commands::ping(),
            commands::stop(),
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
        pre_command : |ctx| {
            Box::pin(async move {
                match ctx {
                    Context::Application(_) => {info!("Received slash Command from {} in channel {}: /{}",
                        ctx.author().name,
                        ctx.channel_id().name(ctx.cache().unwrap()).await.unwrap_or_else(|| {"Unknown".to_string()}),
                        ctx.invoked_command_name());}
                    Context::Prefix(_) => {}
            }
            })
        },
        event_handler: |ctx, event, framework, data| {
            Box::pin(async move {
                #[allow(unused_variables)] let temp = ctx;
                #[allow(unused_variables)] let temp2: poise::dispatch::FrameworkContext<Data, _> = framework;
                #[allow(unused_variables)] let temp4 = data;
                if let poise::event::Event::Message { new_message } = event {
                    if new_message.content.starts_with(framework
                        .options()
                        .prefix_options
                        .prefix
                        .as_ref()
                        .unwrap()
                        .as_str()) {
                        info!("Received message from {} in channel {}: {}",
                                new_message.author.name,
                                new_message.channel_id.name(ctx.cache().unwrap())
                                    .await.unwrap_or_else(|| {"Unknown".to_string()}),
                                new_message.content);
                    }
                } else {
                    //info!("{:?}", event.name());
                }
                Ok(())
            })
        },
        ..Default::default()
    }).token(xml::get_bot_token()).intents(
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
    ).setup(|ctx, _ready, _framework| {
        Box::pin(async move {
            info!("Logged in as {}", _ready.user.name);
            ctx.set_activity(serenity::Activity::watching("downloads")).await;
            Ok(Data {})
        })
    });

    framework.run().await.expect("Err creating client");
}