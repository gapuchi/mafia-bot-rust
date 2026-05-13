use poise::serenity_prelude::{self as serenity, UserId};
use rand::seq::{IndexedRandom, SliceRandom};

use crate::types::{Context, Error};

#[derive(Debug, Clone)]
pub enum Team {
    Blue,
    Orange,
}

#[derive(Debug, Clone)]
pub enum Role {
    Mafia,
    Villager,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub member: serenity::Member,
    pub team: Team,
    pub role: Role,
}

pub struct Game {
    pub game_master: serenity::UserId,
    pub blue_team: Vec<Player>,
    pub orange_team: Vec<Player>,
}

fn assign_teams(mut members: Vec<serenity::Member>) -> Vec<Player> {
    let mut rng = rand::rng();
    let mid = members.len() / 2;
    members.shuffle(&mut rng);

    let (blue_members, orange_members) = members.split_at(mid);

    blue_members
        .iter()
        .map(|m| Player {
            member: m.clone(),
            team: Team::Blue,
            role: Role::Villager,
        })
        .chain(orange_members.iter().map(|m| Player {
            member: m.clone(),
            team: Team::Orange,
            role: Role::Villager,
        }))
        .collect()
}

impl Game {
    pub fn new(members: Vec<serenity::Member>, game_master: serenity::UserId) -> Self {
        let players = assign_teams(members);
        let (blue_team, orange_team) = players
            .into_iter()
            .partition(|p| matches!(p.team, Team::Blue));

        Game {
            game_master,
            blue_team,
            orange_team,
        }
    }

    pub async fn mafia(
        ctx: Context<'_>,
        members: Vec<serenity::Member>,
        game_master: serenity::UserId,
    ) -> Result<Self, Error> {
        // Inner block forces ThreadRng to drop before the first .await.
        let players: Vec<Player> = {
            let players = assign_teams(members);
            let mut rng = rand::rng();

            let blue_ids: Vec<UserId> = players
                .iter()
                .filter(|p| matches!(p.team, Team::Blue))
                .map(|p| p.member.user.id)
                .collect();
            let orange_ids: Vec<UserId> = players
                .iter()
                .filter(|p| matches!(p.team, Team::Orange))
                .map(|p| p.member.user.id)
                .collect();

            let mafia_selection = if players.len() > 6 {
                vec![
                    *blue_ids.choose(&mut rng).unwrap(),
                    *orange_ids.choose(&mut rng).unwrap(),
                ]
            } else {
                vec![players.choose(&mut rng).unwrap().member.user.id]
            };

            players
                .into_iter()
                .map(|p| {
                    let role = if mafia_selection.contains(&p.member.user.id) {
                        Role::Mafia
                    } else {
                        Role::Villager
                    };
                    Player { role, ..p }
                })
                .collect()
        };

        for p in &players {
            let c = p.member.user.create_dm_channel(ctx.http()).await?;
            c.say(
                ctx.http(),
                format!("You are {:#?} on the {:#?} team!", p.role, p.team),
            )
            .await?;
        }

        let (blue_team, orange_team) = players
            .into_iter()
            .partition(|p| matches!(p.team, Team::Blue));

        Ok(Game {
            game_master,
            blue_team,
            orange_team,
        })
    }

    pub fn players(&self) -> Vec<&Player> {
        self.blue_team
            .iter()
            .chain(self.orange_team.iter())
            .collect()
    }
}
