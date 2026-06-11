use std::collections::HashMap;

use poise::serenity_prelude::{self as serenity, Member};
use serenity::Mentionable;

use crate::game::Game;
use crate::types::Error;
use crate::voting::Voting;

const NUMBER_EMOJIS: [&str; 10] = ["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "0️⃣"];
pub const REVEAL_ROLE_BUTTON_ID: &str = "mafia_reveal_role";

pub fn reveal_role_button() -> serenity::CreateActionRow {
    serenity::CreateActionRow::Buttons(vec![serenity::CreateButton::new(REVEAL_ROLE_BUTTON_ID)
        .label("Reveal My Role")
        .style(serenity::ButtonStyle::Primary)])
}

pub struct GameMessage {
    pub message_id: serenity::MessageId,
}

impl GameMessage {
    pub async fn handle_component_interaction(
        &self,
        ctx: &serenity::Context,
        interaction: &serenity::ComponentInteraction,
        game: &Game,
    ) -> Result<(), Error> {
        if interaction.data.custom_id != REVEAL_ROLE_BUTTON_ID {
            return Ok(());
        }

        if interaction.message.id != self.message_id {
            return Ok(());
        }

        let response = match game.player_for(interaction.user.id) {
            Some(player) => serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(player.role_embed())
                    .ephemeral(true),
            ),
            None => serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("You're not in this game.")
                    .ephemeral(true),
            ),
        };

        interaction.create_response(ctx, response).await?;
        Ok(())
    }

    pub async fn handle_add_reaction(
        &self,
        ctx: &serenity::Context,
        reaction: &serenity::Reaction,
        game: &Game,
    ) -> Result<Option<Voting>, Error> {
        let Some(member) = reaction.member.as_ref() else {
            return Ok(None);
        };

        if member.user.id != game.game_master {
            reaction.delete(ctx).await?;
            return Ok(None);
        }

        let serenity::ReactionType::Unicode(emoji) = &reaction.emoji else {
            reaction.delete(ctx).await?;
            return Ok(None);
        };

        let player_to_vote_mentions = match emoji.as_str() {
            "🔷" => &game.orange_team,
            "🔶" => &game.blue_team,
            _ => {
                reaction.delete(ctx).await?;
                return Ok(None);
            }
        };

        let map: Vec<(&'static str, &Member)> = player_to_vote_mentions
            .iter()
            .enumerate()
            .map(|(i, p)| (NUMBER_EMOJIS[i % 10], &p.member))
            .collect();

        let embed = map.iter().fold(
            serenity::CreateEmbed::default().title("Vote Who's Mafia!"),
            |embed, (emoji, member)| embed.field(*emoji, member.mention().to_string(), true),
        );

        let message = reaction
            .channel_id
            .send_message(
                ctx.http.clone(),
                serenity::CreateMessage::new().embed(embed),
            )
            .await?;

        for (emoji, _) in &map {
            message
                .react(
                    ctx.http.clone(),
                    serenity::ReactionType::Unicode(emoji.to_string()),
                )
                .await?;
        }

        Ok(Some(Voting {
            message_id: message.id,
            population: game.players().len(),
            number_to_member: map
                .into_iter()
                .map(|(emoji, member)| (emoji.to_string(), member.user.id))
                .collect(),
            votes: HashMap::new(),
            is_done: false,
        }))
    }
}
