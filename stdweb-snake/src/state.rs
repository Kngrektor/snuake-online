use stdweb::traits::*;
use stdweb::web::IEventTarget;
use stdweb::web::WebSocket;
use stdweb::web::event::{
    KeyDownEvent,
    SocketOpenEvent,
    SocketCloseEvent,
    SocketErrorEvent,
    SocketMessageEvent,
};

extern crate serde;
use serde_json;

use std::cell::RefCell;
use std::rc::Rc;

use snuake_shared::*;

use saas::util::*;
use saas::state::*;
use saas::entity::*;

use std::collections::VecDeque;

// ++++++++++++
// + AppState +
// ++++++++++++

pub trait AppState {
    fn init(&mut self);

    fn should_tick(&mut self, curr_ms: u64) -> bool;

    fn tick(&mut self);

    fn input(&mut self, ev: KeyDownEvent);

    fn get_grid_data(&mut self) -> Option<GridData>;

    fn give_direction(&mut self, dir: Direction);
}

pub type AppStatePtr = Rc<RefCell<Box<AppState>>>;

// ++++++++++++++++
// + OfflineState +
// ++++++++++++++++

pub struct OfflineState {
    is_running: bool,
    snake_id: SnakeID,
    game_state: GameState,
    wait_ms: u64,
    prev_ms: u64,
}

impl OfflineState {
    pub fn new(rows: u32, cols: u32) -> AppStatePtr {
        let mut game_state = GameState::builder()
            .with_dimensions(rows as usize, cols as usize)
            .build();

        let snake_id = game_state.add_snake().unwrap();

        let st = OfflineState {
            is_running: false,
            snake_id: snake_id,
            game_state: game_state,
            wait_ms: 125,
            prev_ms: 0,
        };

        Rc::new(RefCell::new(Box::new(st)))
    }
}

impl AppState for OfflineState {
    fn init(&mut self) { self.is_running = true; }

    fn should_tick(&mut self, curr_ms: u64) -> bool {
        if self.is_running && self.prev_ms + self.wait_ms <= curr_ms {
            self.prev_ms = curr_ms;
            true
        } else {
            false
        }
    }

    fn tick(&mut self) { self.game_state.tick(); }

    fn input(&mut self, ev: KeyDownEvent) {
        match ev.key().as_ref() {
            "p" => { self.game_state.add_snake().unwrap(); },
            _ => (),
        }
    }

    fn get_grid_data(&mut self) -> Option<GridData> {
        Some(self.game_state.get_grid_data())
    }

    fn give_direction(&mut self, dir: Direction) {
        let id = self.snake_id;
        self.game_state.give_direction(id, dir).unwrap();
    }
}

// +++++++++++++++
// + OnlineState +
// +++++++++++++++

#[derive(Debug, Clone, Copy)]
enum State {
    Live,
    Connecting,
}

pub struct OnlineState {
    state: State,
    snake_id: Option<SnakeID>,
    grid_data: Option<GridData>,
    sock: Option<WebSocket>,
    msgs: Rc<RefCell<VecDeque<ServerMsg>>>
}

impl OnlineState {
    pub fn new() -> AppStatePtr {
        let st = OnlineState {
            state: State::Connecting,
            snake_id: None,
            grid_data: None,
            sock: None,
            msgs: Rc::new(RefCell::new(VecDeque::new()))
        };

        Rc::new(RefCell::new(Box::new(st)))
    }
}

impl AppState for OnlineState {
    fn init(&mut self) {
        // connect
        let sock = WebSocket::new("ws://127.0.0.1:8080")
            .map_err(|err| {
            console!(log, "error @ init: WebSocket::new()");
            console!(log, "{:?}", err);
        }).unwrap();

        sock.add_event_listener(|ev: SocketOpenEvent| {
            console!(log, "Socket open!");
            console!(log, "{:?}", ev);
        });

        sock.add_event_listener(|ev: SocketCloseEvent| {
            console!(log, "Socket closed!");
            console!(log, "{:?}", ev);
        });

        sock.add_event_listener(|ev: SocketErrorEvent| {
            console!(log, "Socket error!");
            console!(log, "{:?}", ev);
        });

        sock.add_event_listener({
            let msgs = self.msgs.clone();
            move |ev: SocketMessageEvent| {
                let msgs = &mut msgs.borrow_mut();

                ev.data().into_text().map(|msg| {
                    let msg = msg.as_bytes();
                    serde_json::from_slice(msg)
                        .map(|msg| msgs.push_back(msg))
                        .unwrap();
                });

                console!(log, "{:?}", &ev);
            }
        });

        self.sock = Some(sock);
    }

    fn should_tick(&mut self, _curr_ms: u64) -> bool { true }

    fn tick(&mut self) {
        let st = self.state;
        let msg = self.msgs.borrow_mut().pop_front();

        msg.map(|msg|
            match st {
                State::Connecting => {
                    match msg {
                        ServerMsg::NewID(id) => {
                            self.snake_id = Some(id);
                            self.state = State::Live;
                        },

                        _ => (),
                    }
                },

                State::Live => {
                    match msg {
                        ServerMsg::GridData(gd) => {
                            self.grid_data = Some(gd);
                        },

                        _ => (),
                    }
                }
            }
        );
    }

    fn input(&mut self, _ev: KeyDownEvent) {  }

    fn get_grid_data(&mut self) -> Option<GridData> {
        self.grid_data.take()
    }

    fn give_direction(&mut self, dir: Direction) {
        let st = self.state;

        match st {
            State::Live => {
                let cmd = UserCmd::Direction(dir);
                let msg = ClientMsg::UserCmd(cmd);
                let s = serde_json::to_string_pretty(&msg).unwrap();
                self.sock.as_ref().map(|sock| sock.send_text(&s));
            },

            _ => (),
        }
    }
}

