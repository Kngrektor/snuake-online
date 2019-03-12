extern crate tokio_tungstenite;

use std::net::SocketAddr;

use futures::Sink;
use tokio::executor::Spawn;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::UnboundedSender as mpscUS;

use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::core::ConnMap;
use snuake_shared::*;

pub fn conn(
    s: TcpStream,
    addr: SocketAddr,
    core_s: mpscUS<(SocketAddr, ClientMsg)>,
    conns: ConnMap,
) -> Spawn {
    let a = accept_async(s).and_then(move |ws| {
        println!("New WebSocket connection: {}", addr);
        let (ws_s, ws_r) = ws.split();
        let (ch_s, ch_r) = tokio::sync::mpsc::unbounded_channel();
        conns.lock().unwrap().insert(addr, ch_s);

        let addr_inner = addr.clone();
        let ws_r = ws_r
            .map_err(|_| ())
            .filter_map(move |msg| {
                if let Message::Text(s) = msg {
                    let msg = serde_json::from_str(&s).unwrap();
                    Some((addr_inner, msg))
                } else {
                    None
                }
            })
            .forward(core_s.sink_map_err(|_| ()))
            .map(|_| ())
            .map_err(|_| ());

        let ws_s = ch_r
            .map_err(|_| ())
            .filter_map(|msg| {
                let msg = serde_json::to_string(&msg).ok()?;
                Some(Message::Text(msg))
            })
            .forward(ws_s.sink_map_err(|_| ()))
            .map(|_| ());

        let conn = ws_r.select(ws_s).then(move |_| {
            conns.lock().unwrap().remove(&addr);
            println!("Connection {} closed.", addr);
            Ok(())
        });

        tokio::spawn(conn);
        Ok(())
    });
    tokio::spawn(a.map_err(|e| {
        println!("Error during the websocket handshake occurred: {}", e);
    }))
}
