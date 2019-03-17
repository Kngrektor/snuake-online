use stdweb::traits::*;
use stdweb::web::{event::KeyDownEvent, IEventTarget};

use std::cell::RefCell;
use std::rc::Rc;

use saas::util::*;
use saas::state::*;
use saas::entity::*;

// +++++++++
// + State +
// +++++++++

pub trait State {
    fn init(&mut self);

    fn should_tick(&mut self, curr_ms: u64) -> bool;

    fn tick(&mut self);

    fn input(&mut self, ev: KeyDownEvent);

    fn get_grid_data(&self) -> GridData;

    fn give_direction(&mut self, dir: Direction);
}

pub type StatePtr = Rc<RefCell<Box<State>>>;

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
    pub fn new(rows: u32, cols: u32) -> StatePtr {
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

impl State for OfflineState {
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

    fn get_grid_data(&self) -> GridData { self.game_state.get_grid_data() }

    fn give_direction(&mut self, dir: Direction) {
        let id = self.snake_id;
        self.game_state.give_direction(id, dir).unwrap();
    }
}

// +++++++++++++++
// + OnlineState +
// +++++++++++++++





