#[macro_use]
extern crate stdweb;

use stdweb::traits::*;
use stdweb::web::{event::KeyDownEvent, IEventTarget};

use std::rc::Rc;
use std::borrow::Borrow;

mod canvas;
use crate::canvas::*;

mod state;
use crate::state::*;

mod img_placeholder;
mod resource_loader;
use crate::resource_loader::*;

use saas::util::*;
use saas::state::*;
use saas::entity::*;


const BKG_COLOR: &str = "black";

// ++++++++
// + Draw +
// ++++++++

trait Draw {
    fn draw(&self, gc: &GridCanvas, avatars: &AvatarMap);
}

impl Draw for GridData {
    fn draw(&self, gc: &GridCanvas, avatars: &AvatarMap) {
        let mut it = self.tags.iter();

        for i in 0..self.rows {
            for j in 0..self.cols {
                let i = i as u32;
                let j = j as u32;
                let tag = it.next().unwrap();

                let color = match tag {
                    Tag { kind: Kind::Prop, id: 0 } => "green",
                    Tag { kind: Kind::Prop, id: _ } => "red",
                    Tag { kind: Kind::SnakeBody, id: 0 } => "cyan",
                    Tag { kind: Kind::SnakeBody, id: _ } => "magenta",
                    _ => BKG_COLOR,
                };

                gc.draw_rect_at(color, j, i);
            }
        }

        let mut it = self.tags.iter();

        for i in 0..self.rows {
            for j in 0..self.cols {
                let i = i as u32;
                let j = j as u32;
                let tag = it.next().unwrap();

                if let Tag { kind: Kind::SnakeHead, id } = tag {
                    gc.draw_img_at(avatars.get_img(&id), j, i);
                }
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

fn game_loop(
    state: AppStatePtr,
    canvas: Rc<Canvas>,
    avatars: Rc<AvatarMap>,
    curr_ms: u64)
{
    {
        let mut st = state.borrow_mut();

        if st.should_tick(curr_ms) { st.tick(); }

        // draw
        st.get_grid_data().map(|gd| {
            let canvas = canvas.clone();
            let canvas = canvas.grid_canvas(gd.rows as u32, gd.cols as u32);
            let avatars = avatars.clone();
            gd.draw(&canvas, avatars.borrow());
        });
    }

    stdweb::web::window().request_animation_frame(move |time| {
        game_loop(state.clone(), canvas.clone(), avatars.clone(), time as u64)
    });
}

fn main() {
    stdweb::initialize();

    let state = OfflineState::new(20, 20);
    let canvas = Rc::new(Canvas::new("#canvas"));
    let avatars = Rc::new(ImageLoader::<SnakeID>::user_avatars());
    // let state = OnlineState::new();

    on_key_down(&state);
    game_loop(state.clone(), canvas, avatars, 0);

    state.borrow_mut().init();

    stdweb::event_loop();
}
