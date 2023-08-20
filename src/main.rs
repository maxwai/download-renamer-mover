mod commands;

use poise::serenity_prelude as serenity;
use std::time::Duration;

/// User data, which is stored and accessible in all command invocations
pub struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::help(),
                commands::reload_slash(),
                commands::ping(),
                commands::stop(),
            ],
            allowed_mentions: Some({
                let mut f = serenity::CreateAllowedMentions::default();
                // Only support direct user pings by default
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
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(|ctx, _ready, _framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                ctx.set_activity(serenity::Activity::watching("downloads")).await;
                //poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
