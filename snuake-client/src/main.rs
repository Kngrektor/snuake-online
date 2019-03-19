#[macro_use]
extern crate stdweb;

use stdweb::traits::*;
use stdweb::web::{event::KeyDownEvent, IEventTarget};

use std::rc::Rc;

mod canvas;
use crate::canvas::*;

mod state;
use crate::state::*;

mod resource_loader;

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
                gc.draw_rect_at(color, j as u32, i as u32);
            }
        }
    }
}

fn on_key_down(state: &AppStatePtr) {
    stdweb::web::document().add_event_listener({
        let state = state.clone();
        move |ev: KeyDownEvent | {
            let state = &mut state.borrow_mut();
            match ev.key().as_ref() {
                "w" => state.give_direction(Direction::Up),
                "a" => state.give_direction(Direction::Left),
                "s" => state.give_direction(Direction::Down),
                "d" => state.give_direction(Direction::Right),
                _ => state.input(ev),
            }
        }
    });
}

fn game_loop(state: AppStatePtr, canvas: Rc<Canvas>, curr_ms: u64) {
    {
        let mut st = state.borrow_mut();

        if st.should_tick(curr_ms) {
            // tick
            st.tick();

            // draw
            st.get_grid_data().map(|gd| {
                let canvas = canvas.clone();
                let canvas = canvas.grid_canvas(gd.rows as u32, gd.cols as u32);
                canvas.clear("black");
                gd.draw(&canvas);
            });
        }
    }

    stdweb::web::window().request_animation_frame(move |time| {
        game_loop(state.clone(), canvas.clone(), time as u64)
    });
}

fn main() {
    stdweb::initialize();

    let canvas = Canvas::new("#canvas");
    let canvas = Rc::new(canvas);
    let state = OfflineState::new(20, 20);
    // let state = OnlineState::new();

    on_key_down(&state);
    game_loop(state.clone(), canvas, 0);

    state.borrow_mut().init();

    stdweb::event_loop();
}
