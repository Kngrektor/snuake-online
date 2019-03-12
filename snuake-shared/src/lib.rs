extern crate serde;
use serde::{Deserialize, Serialize};
// This should be put in game
#[derive(Debug, Serialize, Deserialize)]
pub enum UserCommand {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

/// Comes in from net
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Ping(usize),
    Authenticate,
    CCmd(String),
    UCmd(UserCommand),
}

/// Goes out to the net
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    Pong(usize),
    Game(String),
}
