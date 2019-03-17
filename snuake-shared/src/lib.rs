extern crate serde;
use serde::{Deserialize, Serialize};

use saas::util::Direction;
use saas::entity::SnakeID;

#[derive(Debug, Serialize, Deserialize)]
pub enum UserCommand {
    Direction(Direction),
}

// From clients to server
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Ping(usize),
    Authenticate,
    CCmd(String),
    UCmd(UserCommand),
}

// From server to clients
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    Pong(usize),
    Game(String),
    NewID(SnakeID),
}
