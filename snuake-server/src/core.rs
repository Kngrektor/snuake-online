use snuake_shared::*;

use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use std::net::SocketAddr;

use futures::prelude::*;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender as mpscUS;

pub type ConnMap = Arc<Mutex<HashMap<SocketAddr, mpscUS<ServerMsg>>>>;

pub fn core() -> (mpscUS<(SocketAddr, ClientMsg)>, ConnMap, impl Future<Item = (), Error = ()>) {
    let conns: ConnMap = Arc::new(Mutex::new(HashMap::new()));

    let (core_s, core_r) = mpsc::unbounded_channel();

    (core_s, conns.clone(),
        core_r
        .map_err(|_|())
        .for_each(core_inner(conns)))
}

fn core_inner(conns: ConnMap) -> impl FnMut((SocketAddr, ClientMsg)) -> Result<(), ()> {
    move |(addr, msg)|{
        match msg {
            ClientMsg::CCmd(s) => {
                println!("CCmd from {:?}: {:?}", &addr, s);
            }
            ClientMsg::Ping(u) => {
                println!("Ping from {:?}: {:?}", &addr, u);
                let sender = conns.lock().unwrap().get(&addr).unwrap().clone();
                tokio::spawn(sender
                    .send(ServerMsg::Pong(u))
                    .map(|_|())
                    .map_err(|_|()));
            }
            /*ClientMsg::UCmd(u) => {
                println!("Chat from {:?}: {:?}", &addr, u);
                for (_, s) in conns.lock().unwrap().iter() {
                    tokio::spawn(s.clone()
                        .send(ServerMsg::Chat(u.clone()))
                        .map(|_|())
                        .map_err(|_|()));
                }
            }*/
            _ => panic!("Not implemented :(")
        }
        Ok(())
    }
}