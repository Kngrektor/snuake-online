extern crate serde;
use serde::{Deserialize, Serialize};

use saas::util::Direction;
pub use saas::entity::SnakeID;
use saas::state::GameData;

pub const TICKS_PER_SECOND: u64 = 8;

#[derive(Debug, Serialize, Deserialize)]
pub enum UserCmd {
    Direction(Direction),
}

// From clients to server
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Ping(usize),
    Join,
    ConsoleCmd(String),
    UserCmd(UserCmd),
}

// From server to clients
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    Pong(usize),
    GameData(GameData),
    NewID(SnakeID),
}
