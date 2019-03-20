extern crate tokio_tungstenite;

use futures::Sink;
use futures::future::FutureResult;

use tokio::executor::Spawn;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::UnboundedSender as mpscUS;

use tokio_tungstenite::{accept_async};
use tokio_tungstenite::WebSocketStream as WSStream;
use tokio_tungstenite::tungstenite::protocol::Message;

use snuake_shared::*;

use crate::core::Event;

pub fn new(s: TcpStream, core_s: mpscUS<Event>) -> Spawn {
    let addr = s.peer_addr().unwrap();

    let ws = move |ws: WSStream<TcpStream>| -> FutureResult<(), ()> {
        println!("New WebSocket connection: {}", addr);
        let (ws_send, ws_recv) = ws.split();
        let (ch_send, ch_recv) = tokio::sync::mpsc::unbounded_channel();

        // On open + WebSocket msgs + On close -> Core

        // WebSocket opened
        let opened = future::ok(Event::Opened(addr.clone(), ch_send))
            .into_stream();
        
        // WebSocket closed
        let closed = future::ok(Event::Closed(addr.clone()))
            .into_stream();

        // WebSocket messages
        let messages = ws_recv
            .map_err(|_| ())
            .filter_map(|msg| {
                if let Message::Text(s) = msg {
                    let msg = serde_json::from_str(&s).unwrap();
                    Some(msg)
                } else {
                    None
                }
            })
            .map({ let addr = addr.clone(); move |msg|{
                match msg {
                    ClientMsg::Join  => Event::Join(addr),
                    ClientMsg::ConsoleCmd(s) => Event::CCmd(Some(addr), s),
                    ClientMsg::UserCmd(ucmd) => Event::UCmd(addr, ucmd),
                    ClientMsg::Ping(u) => Event::Ping(addr, u)
                }
            }});

        // Merging
        let client_to_core = opened
            .chain(messages)
            .chain(closed)
            .forward(core_s.sink_map_err(|_| panic!("Failed to send to core")))
            .map(|_| ())
            .map_err(|_| ());

        // Forwards messages from core to client
        let core_to_client = ch_recv
            .map_err(|_| ())
            .filter_map(|msg| {
                let msg = serde_json::to_string(&msg).ok()?;
                Some(Message::Text(msg))
            })
            .forward(ws_send.sink_map_err(|_| ()))
            .map(|_| ());

        let conn = client_to_core.select(core_to_client)
            .then(move |res| {
                if let Err(_) = res {
                    println!("Rip {}, for they died at this moment", addr);
                } 
                println!("Connection {} closed.", addr);
                future::ok(())
            });

        tokio::spawn(conn);
        future::ok(())
    };

    let accept = accept_async(s)
        .map_err(|e| {
            println!("Error during the websocket handshake occurred: {}", e);
        })
        .and_then(ws);
    tokio::spawn(accept)
}
