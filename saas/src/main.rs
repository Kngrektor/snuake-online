#![feature(drain_filter)]

use std::env;

extern crate saas;
use saas::entity::*;
use saas::state::*;
use saas::util::*;

extern crate getch;

extern crate colored;
use colored::*;

// ++++++++
// + Draw +
// ++++++++

trait Draw {
    fn draw(&self);
}

impl Draw for Tag {
    fn draw(&self) {
        let s = match self {
            Tag {
                kind: Kind::None,
                id: _,
            } => " ".white(),
            Tag {
                kind: Kind::Prop,
                id: 0,
            } => "+".green(),
            Tag {
                kind: Kind::Prop,
                id: _,
            } => "+".red(),
            Tag {
                kind: Kind::SnakeHead,
                id: 0,
            } => "*".cyan(),
            Tag {
                kind: Kind::SnakeBody,
                id: 0,
            } => "o".cyan(),
            Tag {
                kind: Kind::SnakeHead,
                id: _,
            } => "*".magenta(),
            Tag {
                kind: Kind::SnakeBody,
                id: _,
            } => "o".magenta(),
        };

        print!("{}", s);
    }
}

impl Draw for GridData {
    fn draw(&self) {
        let mut it = self.tags.iter();

        for _ in 0..self.rows {
            for _ in 0..self.cols {
                it.next().unwrap().draw();
            }

            println!("");
        }
    }
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let getch = getch::Getch::new();

    let mut st = State::builder()
        // .with_dimensions(5, 5)
        .with_prop_spawn_timer(Timer::new(3))
        .build();

    let player_id = match st.add_snake() {
        Some(id) => id,
        None => panic!("error @ main: None"),
    };

    loop {
        println!("score: {}!", st.get_score(player_id).unwrap());

        match getch.getch() {
            Ok(x) if x as char == 'w' => {
                st.give_direction(player_id, Direction::Up).unwrap()
            }

            Ok(x) if x as char == 'a' => {
                st.give_direction(player_id, Direction::Left).unwrap()
            }

            Ok(x) if x as char == 's' => {
                st.give_direction(player_id, Direction::Down).unwrap()
            }

            Ok(x) if x as char == 'd' => {
                st.give_direction(player_id, Direction::Right).unwrap()
            }

            Ok(x) if x as char == 'p' => {
                st.add_snake();
                ()
            }

            Ok(x) if x as char == 'q' => break,

            Ok(_) => (),
            Err(_) => (),
        }

        st.tick();

        let gd = st.get_grid_data();

        for _ in 0..gd.cols {
            print!("-");
        }
        println!("");
        gd.draw();
        for _ in 0..gd.cols {
            print!("-");
        }
        println!("");
    }
}
