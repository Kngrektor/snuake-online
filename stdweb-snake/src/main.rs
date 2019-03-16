#[macro_use]
extern crate stdweb;

mod canvas;
use crate::canvas::Canvas;

fn main() {
    stdweb::initialize();

    let canvas = Canvas::new("#canvas");
    let gc = canvas.grid_canvas(20, 20);

    gc.clear("black");
    gc.draw_at(0, 0, "red");
    gc.draw_at(1, 1, "orange");
    gc.draw_at(19, 19, "blue");

    stdweb::event_loop();
}
