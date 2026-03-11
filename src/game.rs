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

impl Game {
    pub async fn new(
        ctx: Context<'_>,
        mut members: Vec<serenity::Member>,
        game_master: serenity::UserId,
    ) -> Result<Self, Error> {
        // In block to force ThreadRng to drop.
        let players = {
            let mut rng = rand::rng();
            let mid = members.len() / 2;

            members.shuffle(&mut rng);

            let user_ids = members.iter().map(|m| m.user.id).collect::<Vec<UserId>>();
            let (b, o) = user_ids.split_at(mid);

            let mafia_selection = if members.len() > 6 {
                vec![*b.choose(&mut rng).unwrap(), *o.choose(&mut rng).unwrap()]
            } else {
                vec![members.choose(&mut rng).unwrap().user.id]
            };

            members
                .into_iter()
                .map(|member| {
                    let team = if b.contains(&member.user.id) {
                        Team::Blue
                    } else {
                        Team::Orange
                    };

                    let role = if mafia_selection.contains(&member.user.id) {
                        Role::Mafia
                    } else {
                        Role::Villager
                    };

                    return Player { member, team, role };
                })
                .collect::<Vec<Player>>()
        };

        for p in &players {
            let c = p.member.user.create_dm_channel(ctx.http()).await?;
            c.say(
                ctx.http(),
                format!("You are {:#?} on the {:#?} team!", p.role, p.team),
            )
            .await?;
        }

        let (blue_team, orange_team): (Vec<Player>, Vec<Player>) = players
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
