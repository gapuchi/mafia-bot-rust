mod game;

use poise::serenity_prelude as serenity;
use serenity::Mentionable;
use std::sync::Arc;
use tokio::sync::Mutex;

struct Data {
    game: Arc<Mutex<Option<game::Game>>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::ReactionAdd { add_reaction, .. } => {
            let message_id = add_reaction.message_id;
            let game = data.game.lock().await;
            let x = game.unwrap();
            if message_id != x.message_id {
                return Ok(());
            }
        }
        _ => {}
    }
    Ok(())
}

#[poise::command(prefix_command, slash_command, subcommands("new"))]
async fn game(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn new(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be in a guild")?;

    let voicehannel_id = {
        let guild = ctx.guild().ok_or("Could not find guild")?;
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

    let mut game = game::Game::new(members, serenity::MessageId::new(1));

    let blue_team: Vec<String> = game
        .players
        .iter()
        .filter(|p| matches!(p.team, game::Team::Blue))
        .map(|p| p.member.mention().to_string())
        .collect();

    let orange_team: Vec<String> = game
        .players
        .iter()
        .filter(|p| matches!(p.team, game::Team::Orange))
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

    game.message_id = message.id;

    let mut game_lock = ctx.data().game.lock().await;
    *game_lock = Some(game);

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
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("$".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            commands: vec![game(), register()],
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
