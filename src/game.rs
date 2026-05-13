use poise::serenity_prelude::{self as serenity, UserId};
use rand::Rng;
use rand::seq::{IndexedRandom, SliceRandom};

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
    members.shuffle(&mut rng);

    let n = members.len();
    let extra_to_blue = n % 2 == 1 && rng.random_bool(0.5);
    let blue_size = n / 2 + if extra_to_blue { 1 } else { 0 };

    let (blue_members, orange_members) = members.split_at(blue_size);

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

fn select_mafia(
    count: usize,
    blue_ids: &[UserId],
    orange_ids: &[UserId],
    rng: &mut impl Rng,
) -> Vec<UserId> {
    let larger = count.div_ceil(2);
    let smaller = count / 2;

    let (blue_count, orange_count) = match blue_ids.len().cmp(&orange_ids.len()) {
        std::cmp::Ordering::Greater => (larger, smaller),
        std::cmp::Ordering::Less => (smaller, larger),
        std::cmp::Ordering::Equal => {
            if rng.random_bool(0.5) {
                (larger, smaller)
            } else {
                (smaller, larger)
            }
        }
    };

    blue_ids
        .choose_multiple(rng, blue_count)
        .chain(orange_ids.choose_multiple(rng, orange_count))
        .copied()
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

    pub fn mafia(
        members: Vec<serenity::Member>,
        game_master: serenity::UserId,
        mafia_count: Option<u32>,
    ) -> Self {
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

        let count = mafia_count
            .map(|c| c as usize)
            .unwrap_or(if players.len() > 6 { 2 } else { 1 });

        let mafia_selection = select_mafia(count, &blue_ids, &orange_ids, &mut rng);

        let players: Vec<Player> = players
            .into_iter()
            .map(|p| {
                let role = if mafia_selection.contains(&p.member.user.id) {
                    Role::Mafia
                } else {
                    Role::Villager
                };
                Player { role, ..p }
            })
            .collect();

        let (blue_team, orange_team) = players
            .into_iter()
            .partition(|p| matches!(p.team, Team::Blue));

        Game {
            game_master,
            blue_team,
            orange_team,
        }
    }

    pub fn players(&self) -> Vec<&Player> {
        self.blue_team
            .iter()
            .chain(self.orange_team.iter())
            .collect()
    }
}
