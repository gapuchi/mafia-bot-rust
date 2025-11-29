use poise::serenity_prelude as serenity;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub enum Team {
    Blue,
    Orange,
}

#[derive(Debug, Clone)]
pub enum Role {
    Mafia,
    Civilian,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub member: serenity::Member,
    pub team: Team,
    pub role: Role,
}

pub struct Game {
    pub message_id: serenity::MessageId,
    pub players: Vec<Player>,
}

impl Game {
    pub fn new(mut members: Vec<serenity::Member>, message_id: serenity::MessageId) -> Self {
        let mafia_count = if members.len() > 6 { 2 } else { 1 };

        members.shuffle(&mut rand::rng());

        let mid = members.len() / 2;

        let players: Vec<Player> = members
            .into_iter()
            .enumerate()
            .map(|(i, member)| {
                let team = if i < mid { Team::Blue } else { Team::Orange };

                let role = if i == 0 {
                    Role::Mafia
                } else if i == mid && mafia_count == 2 {
                    Role::Mafia
                } else {
                    Role::Civilian
                };

                Player { member, team, role }
            })
            .collect();

        Game {
            players,
            message_id,
        }
    }
}
