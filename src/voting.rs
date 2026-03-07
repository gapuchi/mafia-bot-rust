use poise::serenity_prelude::{self as serenity, CacheHttp, Mentionable};
use std::collections::HashMap;

use crate::{
    game::{Game, Role},
    types::Error,
};

pub struct Voting {
    pub message_id: serenity::MessageId,
    pub number_to_member: HashMap<String, serenity::UserId>,
    pub votes: HashMap<serenity::UserId, serenity::UserId>,
    pub is_done: bool,
}

impl Voting {
    pub async fn handle_add_reaction(
        &mut self,
        ctx: &serenity::Context,
        reaction: &serenity::Reaction,
        game: &Game,
    ) -> Result<(), Error> {
        let Some(voter) = reaction.user_id else {
            return Ok(());
        };

        if self.is_done {
            reaction.delete(ctx).await?;
            return Ok(());
        }

        let serenity::ReactionType::Unicode(emoji) = &reaction.emoji else {
            reaction.delete(ctx).await?;
            return Ok(());
        };

        let Some(vote) = self.number_to_member.get(emoji) else {
            reaction.delete(ctx).await?;
            return Ok(());
        };

        if let Some(old_vote) = self.votes.get(&voter) {
            let old_entry = self
                .number_to_member
                .iter()
                .filter(|(_, vote)| *vote == old_vote)
                .map(|(emoji, _)| emoji)
                .next();

            let Some(old_emoji) = old_entry else {
                return Ok(());
            };

            let message = reaction.message(ctx.http()).await?;
            message
                .delete_reaction(
                    ctx.http(),
                    reaction.user_id,
                    serenity::ReactionType::Unicode(old_emoji.to_string()),
                )
                .await?;
        }

        self.votes.insert(voter, *vote);

        if self.votes.len() == self.number_to_member.len() {
            self.is_done = true;
            self.reveal_mafia(ctx, reaction, game).await?;
            self.count_votes();
        }

        Ok(())
    }

    pub async fn handle_remove_reaction(
        &mut self,
        reaction: &serenity::Reaction,
    ) -> Result<(), Error> {
        let Some(voter) = reaction.user_id else {
            return Ok(());
        };

        let serenity::ReactionType::Unicode(emoji) = &reaction.emoji else {
            // If they remove an emoji that isn't a valid vote, ignore
            return Ok(());
        };

        if self.number_to_member.get(emoji).is_none() {
            // If they remove an emoji that isn't a valid vote, ignore
            return Ok(());
        };

        self.votes.remove(&voter);

        Ok(())
    }

    fn count_votes(&self) {}

    async fn reveal_mafia(
        &self,
        ctx: &serenity::Context,
        reaction: &serenity::Reaction,
        game: &Game,
    ) -> Result<(), Error> {
        let mafia: Vec<String> = game
            .players
            .iter()
            .filter(|p| matches!(p.role, Role::Mafia))
            .map(|p| p.member.mention().to_string())
            .collect();

        let embed = serenity::CreateEmbed::default()
            .title("Votes Are In! ")
            .field("Mafia", mafia.join("\n"), true);

        reaction
            .channel_id
            .send_message(ctx.http(), serenity::CreateMessage::new().embed(embed))
            .await?;

        Ok(())
    }
}
