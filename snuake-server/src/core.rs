use snuake_shared::*;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Instant, Duration};

use futures::prelude::*;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender as mpscUS;
use tokio::timer::Interval;

pub enum Event {
    Tick,
    Opened(SocketAddr, mpscUS<ServerMsg>),
    Closed(SocketAddr),
    Ping(SocketAddr, usize),
    Join(SocketAddr),
    CCmd(Option<SocketAddr>, String),
    UCmd(SocketAddr, UserCmd),
}

pub fn core() -> (mpscUS<Event>, impl Future<Item = (), Error = ()>) {

    let (core_s, core_r) = mpsc::unbounded_channel();

    let ticker = Interval::new(Instant::now(), Duration::from_millis(125))
        .map(|_|Event::Tick)
        .map_err(|_| ());
        
    let core_r = core_r
        .map_err(|_| ());

    (
        core_s,
        ticker.select(core_r)
            .for_each(core_inner())
    )
}

use saas;

fn core_inner() -> impl FnMut(Event) -> Result<(), ()> {
    let mut connections = HashMap::new();
    let mut snake_ids = HashMap::new();
    let mut snake_game = saas::state::GameState::builder()
            .with_dimensions(20,20)
            .build();

    move |event| {
        match event {
            Event::Opened(addr, ch_s) => {
                println!("Opened conn to {:?}", addr);
                connections.insert(addr, ch_s);
            }
            Event::Closed(addr) => {
                println!("Closed conn to {:?}", addr);
            }
            Event::CCmd(addr, s) => {
                println!("CCmd from {:?}: {:?}", addr, s);
            }
            Event::Ping(addr, u) => {
                println!("Ping from {:?}: {:?}", addr, u);
                let ws_s = connections.get(&addr).unwrap().clone();
                tokio::spawn(
                    ws_s.send(ServerMsg::Pong(u)).map(|_| ()).map_err(|_| ()),
                );
            }
            Event::Join(addr) => {
                println!("Authenticate from {:?}", addr);
                let snake_id = snake_game.add_snake().unwrap();
                let ws_s = connections.get(&addr).unwrap().clone();
                tokio::spawn(
                    ws_s.send(ServerMsg::NewID(snake_id)).map(|_| ()).map_err(|_| ()),
                );
                snake_ids.insert(addr, snake_id);
            }
            Event::UCmd(addr, UserCmd::Direction(dir)) => {
                println!("{:?}: got {:?}", addr, dir);
                if let Some(sid) = snake_ids.get(&addr) {
                    snake_game.give_direction(*sid, dir).unwrap();
                } else {
                    println!("User tried to stuff but has no snakie");
                }
            }
            Event::Tick => {
                snake_game.tick();
                for ws_s in connections.values() {
                    let ws_s = ws_s.clone();
                    let msg = ServerMsg::GridData(snake_game.get_grid_data());
                    let future = ws_s.send(msg).map(|_| ()).map_err(|_| ());
                    tokio::spawn(future);
                }
            }
        }
        Ok(())
    }
}
