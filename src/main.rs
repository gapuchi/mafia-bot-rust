mod event_handler;
mod game;
mod game_message;
mod types;
mod voting;

use poise::serenity_prelude as serenity;
use serenity::Mentionable;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    event_handler::event_handler,
    game_message::GameMessage,
    types::{Context, Data, Error},
};

#[poise::command(prefix_command, slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("pong").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, subcommands("new"))]
async fn game(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, guild_only)]
async fn new(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild_only ensures this is Some");

    let voice_channel_id = {
        let guild = ctx
            .guild()
            .ok_or("Could not find guild (ensure bot has GUILDS intent and was restarted)")?;
        guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
            .ok_or("You must be in a voice channel")?
    };

    let channels = guild_id.channels(ctx).await?;

    let channel = channels
        .get(&voice_channel_id)
        .ok_or("Could not find channel")?;

    let members = channel.members(ctx)?;

    let game = game::Game::new(members, ctx.author().id);

    let blue_team: Vec<String> = game
        .blue_team()
        .iter()
        .map(|p| p.member.mention().to_string())
        .collect();

    let orange_team: Vec<String> = game
        .orange_team()
        .iter()
        .map(|p| p.member.mention().to_string())
        .collect();

    let embed = serenity::CreateEmbed::default()
        .title("New Game!")
        .field("**Blue Team:**", blue_team.join("\n"), true)
        .field("**Orange Team:**", orange_team.join("\n"), true)
        .footer(serenity::CreateEmbedFooter::new(
            " 🔷 Blue won\n🔶 Orange won",
        ));

    let reply = ctx.send(poise::CreateReply::default().embed(embed)).await?;

    let message = reply.into_message().await?;
    message.react(ctx.http(), '🔷').await?;
    message.react(ctx.http(), '🔶').await?;

    *ctx.data().game.lock().await = Some(game);
    *ctx.data().game_message.lock().await = Some(GameMessage {
        message_id: message.id,
    });

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let token =
        std::env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment");

    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
        | serenity::GatewayIntents::GUILD_VOICE_STATES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("$".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            commands: vec![ping(), game(), register()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
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
