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

use crate::streamext::StreamExt;
use crate::core::Event;

pub fn new(s: TcpStream, core_s: mpscUS<Event>) -> Spawn {
    let addr = s.peer_addr().unwrap();

    let ws = move |ws: WSStream<TcpStream>| -> FutureResult<(), ()> {
        println!("New WebSocket connection: {}", addr);
        let (ws_send, ws_recv) = ws.split();
        let (ch_send, ch_recv) = tokio::sync::mpsc::unbounded_channel();

        // WebSocket messages
        let messages = ws_recv
            .end_on_error()
            .filter_map(|msg|{
                if let Message::Text(msg) = msg {
                    Some(serde_json::from_str(&msg).unwrap())
                } else {
                    None
                }
            })
            .map({ let addr = addr.clone(); move |msg|{
                match msg {
                    ClientMsg::Join          => Event::Join(addr),
                    ClientMsg::ConsoleCmd(s) => Event::CCmd(Some(addr), s),
                    ClientMsg::UserCmd(ucmd) => Event::UCmd(addr, ucmd),
                    ClientMsg::Ping(u)       => Event::Ping(addr, u)
                }
            }})
            .map_err(|_|());

        // On open + WebSocket msgs + On close -> Core
        let client_to_core = 
            stream::once(Ok(Event::Opened(addr.clone(), ch_send)))
            .chain(messages)
            .chain(stream::once(Ok(Event::Closed(addr.clone()))))
            .forward(core_s.sink_map_err(|_|()))
            .map(|_| ());


        // Forwards messages from core to client
        let core_to_client = ch_recv
            .map_err(|_| ())
            .filter_map(|msg| {
                let msg = serde_json::to_string(&msg).ok()?;
                Some(Message::Text(msg))
            })
            .forward(ws_send.sink_map_err(|_|()))
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
