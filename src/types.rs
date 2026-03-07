use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{game::Game, game_message::GameMessage, voting::Voting};
pub struct Data {
    pub game: Arc<Mutex<Option<Game>>>,
    pub game_message: Arc<Mutex<Option<GameMessage>>>,
    pub voting_message: Arc<Mutex<Option<Voting>>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
