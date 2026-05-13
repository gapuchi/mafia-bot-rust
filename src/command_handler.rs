use crate::{
    game::Game,
    game_message::GameMessage,
    types::{Context, Error},
};

use poise::serenity_prelude as serenity;
use serenity::Mentionable;

async fn get_voice_channel_members(
    ctx: Context<'_>,
) -> Result<Vec<serenity::Member>, Error> {
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

    Ok(channel.members(ctx)?)
}

async fn post_game(ctx: Context<'_>, game: Game) -> Result<(), Error> {
    let blue_team: Vec<String> = game
        .blue_team
        .iter()
        .map(|p| p.member.mention().to_string())
        .collect();

    let orange_team: Vec<String> = game
        .orange_team
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

pub async fn create_simple_game(ctx: Context<'_>) -> Result<(), Error> {
    let members = get_voice_channel_members(ctx).await?;
    let game = Game::new(members, ctx.author().id);

    let blue_team: Vec<String> = game
        .blue_team
        .iter()
        .map(|p| p.member.mention().to_string())
        .collect();

    let orange_team: Vec<String> = game
        .orange_team
        .iter()
        .map(|p| p.member.mention().to_string())
        .collect();

    let embed = serenity::CreateEmbed::default()
        .title("New Game!")
        .field("**Blue Team:**", blue_team.join("\n"), true)
        .field("**Orange Team:**", orange_team.join("\n"), true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

pub async fn create_game(ctx: Context<'_>, mafia_count: Option<u32>) -> Result<(), Error> {
    let members = get_voice_channel_members(ctx).await?;
    let game = Game::mafia(ctx, members, ctx.author().id, mafia_count).await?;
    post_game(ctx, game).await
}
