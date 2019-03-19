use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::ImageElement;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d};

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

    pub fn draw_rect(&self, color: &str, x: u32, y: u32, w: u32, h: u32) {
        self.ctx.set_fill_style_color(color);

        self.ctx.fill_rect(x.into(), y.into(), w.into(), h.into());
    }

    pub fn draw_img(&self, img: ImageElement, x: u32, y:u32, w:u32, h:u32) {
        self.ctx.draw_image_d(img, x.into(), y.into(), w.into(), h.into());
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
    scaled_width: u32,
    scaled_height: u32,
    cols: u32,
    rows: u32,
}

impl<'a> GridCanvas<'a> {
    pub fn new(canvas: &'a Canvas, rows: u32, cols: u32) -> Self {
        let scaled_width = canvas.width() / cols;
        let scaled_height = canvas.height() / rows;

        Self {
            canvas,
            scaled_width,
            scaled_height,
            cols,
            rows,
        }
    }

    pub fn draw_rect_at(&self, color: &str, x: u32, y: u32) {
        assert!(x < self.cols);
        assert!(y < self.rows);

        let x = x * self.scaled_width;
        let y = y * self.scaled_height;

        self.canvas.draw_rect(
            color,
            x,
            y,
            self.scaled_width,
            self.scaled_height,
        )
    }

    pub fn draw_img_at(&self, img: ImageElement, x: u32, y: u32) {
        self.canvas.draw_img(
            img,
            x,
            y,
            self.scaled_width,
            self.scaled_height,
        )
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

