mod commands;
mod event_handler;
mod game;
mod game_message;
mod types;
mod voting;

use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{event_handler::event_handler, types::Data};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let token =
        std::env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment");

    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
        | serenity::GatewayIntents::GUILD_VOICE_STATES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("$".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            commands: vec![
                commands::ping(),
                commands::game(),
                commands::mafia(),
                commands::register(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    game: Arc::new(Mutex::new(None)),
                    game_message: Arc::new(Mutex::new(None)),
                    voting_message: Arc::new(Mutex::new(None)),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    match client {
        Ok(mut client) => {
            if let Err(e) = client.start().await {
                eprintln!("Error starting client: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error creating client: {}", e);
        }
    }
}
