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

use snuake_shared::*;

struct TickTimer {
    tick_start: u64,
    tick_len: u64,
}

impl TickTimer {
    pub fn new(curr_ms: u64, tps: u64) -> Self {
        TickTimer {
            tick_start: curr_ms,
            tick_len: 1000 / tps,
        }
    }

    pub fn percent_left(&self, curr_ms: u64) -> f64 {
        let elapsed = (curr_ms - self.tick_start) as f64;
        1.0 - elapsed / self.tick_len as f64
    }
}

fn sgn(x: i32) -> i32 {
    if x < 0 { -1 }
    else if 0 < x { 1 }
    else { 0 }
}

fn in_bounds(x: i32, n: i32) -> bool {
    if x < 0 || n <= x { false }
    else { true }
}

// ++++++++
// + Draw +
// ++++++++


fn draw_animated<'a, T : Clone>(
    draw: fn(&GridCanvas<'a>, T, i32, i32, f64, f64),
    gc: &GridCanvas<'a>,
    drawable: T,
    curr_pos: &(u32, u32),
    prev_pos: &(u32, u32),
    translate_factor: f64,
) {
    let (i, j) = *curr_pos;
    let (i2, j2) = *prev_pos;
    let i = i as i32;
    let j = j as i32;
    let i2 = i2 as i32;
    let j2 = j2 as i32;

    let dx = sgn(j2 - j);
    let dy = sgn(i2 - i);

    if j + dx != j2 {
        draw(
            gc,
            drawable.clone(),
            j,
            i,
            -dx as f64 * translate_factor,
            dy as f64 * translate_factor,
        );

        draw(
            gc,
            drawable.clone(),
            j2 + dx,
            i,
            -dx as f64 * translate_factor,
            dy as f64 * translate_factor,
        );
    } else if i + dy != i2 {
        draw(
            gc,
            drawable.clone(),
            j,
            i,
            dx as f64 * translate_factor,
            -dy as f64 * translate_factor,
        );

        draw(
            gc,
            drawable.clone(),
            j,
            i2 + dy,
            dx as f64 * translate_factor,
            -dy as f64 * translate_factor,
        );
    } else {
        draw(
            gc,
            drawable.clone(),
            j,
            i,
            dx as f64 * translate_factor,
            dy as f64 * translate_factor,
        );
    }
}

const BKG_COLOR: &str = "black";

trait Draw {
    fn draw(
        &self,
        gc: &GridCanvas,
        avatars: &AvatarMap,
        translate_factor: f64,
    );
}

impl Draw for GameData {
    fn draw(
        &self,
        gc: &GridCanvas,
        avatars: &AvatarMap,
        translate_factor: f64,
        )
    {
        let rows = self.grid_data.rows as usize;
        let cols = self.grid_data.cols as usize;

        for (_id, came_from) in self.came_from_tails.iter() {
            if let CameFrom::Dummy((curr_pos, prev_pos)) = came_from {
                draw_animated(
                    GridCanvas::draw_rect_at_translated,
                    gc,
                    "yellow",
                    curr_pos,
                    prev_pos,
                    translate_factor,
                );

                let (i, j) = curr_pos;

                gc.draw_rect_at(
                    BKG_COLOR,
                    *j as i32,
                    *i as i32,
                )
            }
        }

        for i in 0 .. rows {
            for j in 0 .. cols {
                let tag = self.grid_data.tags[i][j];
                let i = i as i32;
                let j = j as i32;

                match tag {
                    Tag { kind: Kind::Prop, id: 0 } => Some("green"),
                    Tag { kind: Kind::Prop, id: _ } => Some("red"),
                    Tag { kind: Kind::SnakeBody, id: 0 } => Some("cyan"),
                    Tag { kind: Kind::SnakeBody, id: _ } => Some("magenta"),
                    _ => None,
                }
                .map(|color| gc.draw_rect_at(color, j, i));
            }
        }

        for (_id, came_from) in self.came_from_tails.iter() {
            if let CameFrom::Real((curr_pos, prev_pos)) = came_from {
                draw_animated(
                    GridCanvas::draw_rect_at_translated,
                    gc,
                    "yellow",
                    curr_pos,
                    prev_pos,
                    translate_factor,
                );
            }
        }

        for (id, came_from) in self.came_from_heads.iter() {
            if let CameFrom::Real((curr_pos, prev_pos)) = came_from {
                draw_animated(
                    GridCanvas::draw_img_at_translated,
                    gc,
                    avatars.get_img(&id),
                    curr_pos,
                    prev_pos,
                    translate_factor,
                );
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
    mut tick_timer: TickTimer,
    curr_ms: u64)
{
    {
        let mut st = state.borrow_mut();

        if st.should_tick(curr_ms) {
            st.tick();
            tick_timer = TickTimer::new(curr_ms, TICKS_PER_SECOND);
        }

        // draw
        st.game_data().map(|gd| {
            let canvas = canvas.clone();
            let canvas = canvas.grid_canvas(gd.grid_data.rows, gd.grid_data.cols);
            let avatars = avatars.clone();
            canvas.clear(BKG_COLOR);
            gd.draw(&canvas, avatars.borrow(), tick_timer.percent_left(curr_ms));
        });
    }

    stdweb::web::window().request_animation_frame(move |time| {
        game_loop(
            state.clone(),
            canvas.clone(),
            avatars.clone(),
            tick_timer,
            time as u64,
        )
    });
}

fn main() {
    stdweb::initialize();

    let _state = OnlineState::new();
    let state = OfflineState::new(20, 20);
    let canvas = Rc::new(Canvas::new("#canvas"));
    let avatars = Rc::new(ImageLoader::<SnakeID>::user_avatars());

    on_key_down(&state);

    game_loop(
        state.clone(),
        canvas,
        avatars,
        TickTimer::new(0, TICKS_PER_SECOND),
        0
    );

    state.borrow_mut().init();

    stdweb::event_loop();
}
