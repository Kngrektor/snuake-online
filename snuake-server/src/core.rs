use snuake_shared::*;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::time::{Instant, Duration};

use futures::prelude::*;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender as mpscUS;
use tokio::timer::Interval;

pub type ConnMap = Arc<Mutex<HashMap<SocketAddr, mpscUS<ServerMsg>>>>;

enum Event {
    Tick,
    Net((SocketAddr, ClientMsg))
}

pub fn core() -> (
    mpscUS<(SocketAddr, ClientMsg)>,
    ConnMap,
    impl Future<Item = (), Error = ()>,
) {
    let conns: ConnMap = Arc::new(Mutex::new(HashMap::new()));

    let (core_s, core_r) = mpsc::unbounded_channel();

    let ticker = Interval::new(Instant::now(), Duration::from_millis(125))
        .map_err(|_| ())
        .map(|_|Event::Tick);
    let core_r = core_r
        .map_err(|_| ())
        .map(|net|Event::Net(net));

    (
        core_s,
        conns.clone(),
        ticker.select(core_r)
            .for_each(core_inner(conns))
    )
}

use saas;

fn core_inner(
    conns: ConnMap,
) -> impl FnMut(Event) -> Result<(), ()> {
    let mut sids = HashMap::new();
    let mut game = saas::state::GameState::builder()
            .with_dimensions(20,20)
            .build();
    //let game = Arc::new(Mutex::new(game));
    //let game_inner = game.clone();

    move |event| {
        match event {
            Event::Net((addr, msg)) => {
                println!("{:?}: {:?}", addr, msg);
                match msg {
                    ClientMsg::ConsoleCmd(s) => {
                        println!("CCmd from {:?}: {:?}", addr, s);
                    }
                    ClientMsg::Ping(u) => {
                        println!("Ping from {:?}: {:?}", addr, u);
                        let ws_s = conns.lock().unwrap().get(&addr).unwrap().clone();
                        tokio::spawn(
                            ws_s.send(ServerMsg::Pong(u)).map(|_| ()).map_err(|_| ()),
                        );
                    }
                    ClientMsg::Authenticate => {
                        println!("Authenticate from {:?}", addr);
                        let snake_id = game.add_snake().unwrap();
                        let ws_s = conns.lock().unwrap().get(&addr).unwrap().clone();
                        tokio::spawn(
                            ws_s.send(ServerMsg::NewID(snake_id)).map(|_| ()).map_err(|_| ()),
                        );
                        sids.insert(addr, snake_id);
                    }
                    ClientMsg::UserCmd(UserCmd::Direction(dir)) => {
                        println!("{:?}: got {:?}", addr, dir);
                        if let Some(sid) = sids.get(&addr) {
                            game.give_direction(*sid, dir);
                        } else {
                            println!("User tried to stuff but has no snakie");
                        }
                    }
                    _ => panic!("Not implemented :("),
                }
            }
            Event::Tick => {
                game.tick();
                for ws_s in conns.lock().unwrap().values() {
                    let ws_s = ws_s.clone();
                    let msg = ServerMsg::GridData(game.get_grid_data());
                    let future = ws_s.send(msg).map(|_| ()).map_err(|_| ());
                    tokio::spawn(future);
                }
            }
        }
        Ok(())
    }
}
