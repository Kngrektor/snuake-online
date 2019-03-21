// #[macro_use]
use stdweb;
use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::ImageElement;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d};

use crate::graphics::*;

pub struct Canvas {
    pub canvas: CanvasElement,
    pub ctx: CanvasRenderingContext2d,
}

impl Canvas {
    pub fn new(attr_id: &str) -> Self {
        let canvas: CanvasElement = document()
            .query_selector(attr_id)
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();

        let ctx: CanvasRenderingContext2d = canvas.get_context().unwrap();

        Canvas {
            canvas,
            ctx,
        }
    }

    pub fn width(&self) -> u32 { self.canvas.width() }

    pub fn height(&self) -> u32 { self.canvas.height() }

    pub fn draw_rect(&self, color: &str, x: i32, y: i32, w: i32, h: i32) {
        self.ctx.set_fill_style_color(color);

        self.ctx.fill_rect(x.into(), y.into(), w.into(), h.into());
    }

    pub fn draw_img(&self, img: ImageElement, x: i32, y:i32, w:i32, h:i32) {
        self.ctx
            .draw_image_d(img, x.into(), y.into(), w.into(), h.into())
            .unwrap();
    }

    pub fn translate(&self, x: i32, y: i32) {
        self.ctx.translate(x.into(), y.into());
    }

    pub fn reset_transform(&self) {
        self.ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    }

    pub fn clear(&self, color: &str) {
        self.ctx.set_fill_style_color(color);
        self.ctx.fill_rect(
            0.0,
            0.0,
            self.canvas.width().into(),
            self.canvas.height().into(),
        );
    }
}

pub struct GridCanvas<'a> {
    canvas: &'a Canvas,
    width: i32,
    height: i32,
}

impl<'a> GridCanvas<'a> {
    pub fn new(canvas: &'a Canvas, rows: u32, cols: u32) -> Self {
        let width = canvas.width() as i32 / cols as i32;
        let height = canvas.height() as i32 / rows as i32;

        Self {
            canvas,
            width,
            height,
        }
    }

    pub fn draw_rect_at(&self, color: &str, x: i32, y: i32) {
        let x = x * self.width;
        let y = y * self.height;

        self.canvas.draw_rect(
            color,
            x,
            y,
            self.width,
            self.height,
        )
    }

    pub fn draw_rect_at_translated(
        &self,
        color: &str,
        x: i32,
        y: i32,
        x_factor: f64,
        y_factor: f64,
        )
    {
        let x = x * self.width;
        let y = y * self.height;

        let translate_x = x_factor * self.width as f64;
        let translate_y = y_factor * self.height as f64;
        self.canvas.translate(translate_x as i32, translate_y as i32);

        self.canvas.draw_rect(
            color,
            x,
            y,
            self.width,
            self.height,
        );

        self.canvas.reset_transform();
    }

    pub fn draw_img_at_translated(
        &self,
        image: Image,
        x: i32,
        y: i32,
        x_factor: f64,
        y_factor: f64,
        )
    {
        let x = x * self.width;
        let y = y * self.height;

        let width = (self.width as f64 * image.scale) as i32;
        let height = (self.height as f64 * image.scale) as i32;

        let offset_x: i32 = (width - self.width) / 2;
        let offset_y: i32 = (height - self.height) / 2;

        self.canvas.translate(-offset_x, -offset_y);

        let translate_x = x_factor * self.width as f64;
        let translate_y = y_factor * self.height as f64;
        self.canvas.translate(translate_x as i32, translate_y as i32);

        self.canvas.draw_img(
            image.img,
            x,
            y,
            width,
            height,
        );

        self.canvas.reset_transform();
    }

    pub fn clear(&self, color: &str) {
        self.canvas.clear(color);
    }
}

impl Canvas {
    pub fn grid_canvas<'a>(&'a self, rows: u32, cols: u32) -> GridCanvas<'a> {
        GridCanvas::new(self, rows, cols)
    }
}

