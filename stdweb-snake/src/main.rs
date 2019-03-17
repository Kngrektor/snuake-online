#[macro_use]
extern crate stdweb;

use stdweb::traits::*;
use stdweb::web::{event::KeyDownEvent, IEventTarget};

use std::cell::RefCell;
use std::rc::Rc;

mod canvas;
use crate::canvas::*;

use saas::util::*;
use saas::state::*;
use saas::entity::*;

// ++++++++
// + Draw +
// ++++++++

trait Color {
    fn color(&self) -> &str;
}

impl Color for saas::entity::Tag {
    fn color(&self) -> &str {
        match self {
            Tag { kind: Kind::None, id: _ } => "black",
            Tag { kind: Kind::Prop, id: 0 } => "green",
            Tag { kind: Kind::Prop, id: _ } => "red",
            Tag { kind: Kind::SnakeHead, id: 0 } => "cyan",
            Tag { kind: Kind::SnakeBody, id: 0 } => "cyan",
            Tag { kind: Kind::SnakeHead, id: _ } => "magenta",
            Tag { kind: Kind::SnakeBody, id: _ } => "magenta",
        }
    }
}

trait Draw {
    fn draw(&self, gc: &GridCanvas);
}

impl Draw for GridData {
    fn draw(&self, gc: &GridCanvas) {
        let mut it = self.tags.iter();

        for i in 0..self.rows {
            for j in 0..self.cols {
                let color = it.next().unwrap().color();
                gc.draw_at(i as u32, j as u32, color);
            }
        }
    }
}

struct State {
    snake_id: SnakeID,
    game_state: GameState,
}

const WIDTH: u32 = 20;
const HEIGHT: u32 = 20;

fn init() -> State {
    let mut game_state = GameState::builder()
        .with_dimensions(HEIGHT as usize, WIDTH as usize)
        .build();

    let snake_id = game_state.add_snake().unwrap();

    State {
        snake_id,
        game_state,
    }
}

fn tick(state: &mut State) { state.game_state.tick() }

fn draw(state: &State, grid_canvas: &GridCanvas) {
    grid_canvas.clear("black");
    state.game_state.get_grid_data().draw(&grid_canvas);

}

fn input(state: &Rc<RefCell<State>>) {
    stdweb::web::document().add_event_listener({
        let state = state.clone();
        move |ev: KeyDownEvent | {
            let state: &mut State = &mut state.borrow_mut();
            let game_state: &mut GameState = &mut state.game_state;
            let id = state.snake_id;
            // the directions are all messed up...
            match ev.key().as_ref() {
                "w" => game_state.give_direction(id, Direction::Left).unwrap(),
                "a" => game_state.give_direction(id, Direction::Up).unwrap(),
                "s" => game_state.give_direction(id, Direction::Right).unwrap(),
                "d" => game_state.give_direction(id, Direction::Down).unwrap(),
                "p" => { game_state.add_snake().unwrap(); },
                _ => (),
            }
        }
    });
}

fn game_loop(
    state: Rc<RefCell<State>>,
    canvas: Rc<Canvas>,
    wait_ms: u64,
    prev_ms: u64,
    curr_ms: u64,
    )
{
    let should_tick = prev_ms + wait_ms <= curr_ms;

    if should_tick {
        // borrow state and canvas
        let mut state = state.borrow_mut();
        let canvas = canvas.clone();
        let grid_canvas = canvas.grid_canvas(HEIGHT, WIDTH);

        // tick
        tick(&mut state);

        // draw
        draw(&state, &grid_canvas);
    }

    let prev_ms = if should_tick { curr_ms } else { prev_ms };

    stdweb::web::window().request_animation_frame(move |time| {
        game_loop(state.clone(), canvas.clone(), wait_ms, prev_ms, time as u64)
    });
}

fn main() {
    stdweb::initialize();

    let canvas = Canvas::new("#canvas");
    let canvas = Rc::new(canvas);
    let state = init();
    let state = Rc::new(RefCell::new(state));

    input(&state);
    game_loop(state, canvas, 125, 0, 0);

    stdweb::event_loop();
}
