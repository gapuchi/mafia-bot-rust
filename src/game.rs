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
    pub game_master: serenity::UserId,
    pub players: Vec<Player>,
}

impl Game {
    pub fn new(mut members: Vec<serenity::Member>, game_master: serenity::UserId) -> Self {
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
            game_master,
            players,
        }
    }

    pub fn blue_team(&self) -> Vec<&Player> {
        self.players
            .iter()
            .filter(|p| matches!(p.team, Team::Blue))
            .collect()
    }

    pub fn orange_team(&self) -> Vec<&Player> {
        self.players
            .iter()
            .filter(|p| matches!(p.team, Team::Orange))
            .collect()
    }
}
