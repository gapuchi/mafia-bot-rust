use poise::serenity_prelude::{self as serenity, Reaction};

use crate::types::{Data, Error};

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            handle_add_reaction(ctx, add_reaction, data).await?;
        }
        serenity::FullEvent::ReactionRemove { removed_reaction } => {
            handle_remove_reaction(removed_reaction, data).await?
        }
        _ => {}
    }

    Ok(())
}

async fn handle_add_reaction(
    ctx: &serenity::Context,
    reaction: &Reaction,
    data: &Data,
) -> Result<(), Error> {
    let mut game_guard = data.game.lock().await;
    let Some(game) = game_guard.as_mut() else {
        return Ok(());
    };

    let Some(member) = &reaction.member else {
        return Ok(());
    };

    if member.user.bot {
        return Ok(());
    }

    let message_guard = data.game_message.lock().await;
    let message = message_guard.as_ref();
    if let Some(msg) = message
        && msg.message_id == reaction.message_id
    {
        if let Some(voting_message) = msg.handle_add_reaction(ctx, &reaction, game).await? {
            *data.voting_message.lock().await = Some(voting_message);
        }
    }

    let mut voting_guard = data.voting_message.lock().await;
    let voting_message = voting_guard.as_mut();
    if let Some(msg) = voting_message
        && msg.message_id == reaction.message_id
    {
        msg.handle_add_reaction(ctx, &reaction, game).await?
    }

    Ok(())
}

async fn handle_remove_reaction(reaction: &Reaction, data: &Data) -> Result<(), Error> {
    let mut voting_guard = data.voting_message.lock().await;
    let voting_message = voting_guard.as_mut();

    if let Some(msg) = voting_message
        && msg.message_id == reaction.message_id
    {
        msg.handle_remove_reaction(&reaction).await?
    }

    Ok(())
}
