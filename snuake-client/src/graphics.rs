use stdweb::web::html_element::ImageElement;

use std::str::FromStr;
use std::collections::HashMap;

use snuake_shared::*;
use crate::resource_loader::*;

#[derive(Clone)]
pub struct Image {
    pub img: ImageElement,
    pub scale: f64,
}

pub const BKG_COLOR: &str = "#2f8136";

const SNAKE_HEADS: &[&str] = &[
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f438.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f435.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f43c.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f42f.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f436.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f981.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f43b.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f431.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f439.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f437.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f417-1f464.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f43a.png",
];

const HEAD_SCALING: &[f64] = &[
    1.55,
    1.75,
    1.65,
    1.75,
    1.75,
    1.75,
    1.65,
    1.65,
    1.75,
    1.65,
    1.75,
    1.65,
];

const SNAKE_COLORS: &[&str] = &[
    "#83bf4f",
    "#89664c",
    "#555e63",
    "#f29a2e",
    "#f5d1ac",
    "#e5bc5e",
    "#947151",
    "#4c5359",
    "#f29a2e",
    "#fc97b2",
    "#89664c",
    "#86959c",
];

pub struct SnakeGraphics {
    heads: ImageLoader<SnakeID>,
    scaling: HashMap<SnakeID, f64>,
    colors: HashMap<SnakeID, String>,
}

impl SnakeGraphics {
    pub fn new() -> Self {
        let it = SNAKE_HEADS.iter()
            .enumerate()
            .map(|(i, uri)| (i as SnakeID, *uri));

        let heads = ImageLoader::new(it);

        let scaling = HEAD_SCALING.iter()
            .enumerate()
            .map(|(i, scl)| (i as SnakeID, *scl))
            .collect();

        let colors = SNAKE_COLORS.iter()
            .enumerate()
            .map(|(i, color)| (i as SnakeID, String::from_str(*color).unwrap()))
            .collect();

        SnakeGraphics {
            heads,
            scaling,
            colors,
        }
    }

    pub fn get_head(&self, id: SnakeID) -> Image {
        let img = self.heads.get_img(&id);
        let scale = *self.scaling.get(&id).unwrap();

        Image {
            img,
            scale,
        }
    }

    pub fn get_color(&self, id: SnakeID) -> &str {
        self.colors.get(&id).unwrap()
    }
}

const FRUIT: &[&str] = &[
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f34e.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/1f4a9.png",
];

const FRUIT_SCALING: &[f64] = &[
    1.4,
    1.4,
];

pub struct PropGraphics {
    imgs: ImageLoader<PropID>,
    scaling: HashMap<PropID, f64>,
}

impl PropGraphics {
    pub fn new() -> Self {
        let it = FRUIT.iter()
            .enumerate()
            .map(|(i, uri)| (i as PropID, *uri));

        let imgs = ImageLoader::new(it);

        let scaling = FRUIT_SCALING.iter()
            .enumerate()
            .map(|(i, scl)| (i as PropID, *scl))
            .collect();

        PropGraphics {
            imgs,
            scaling,
        }
    }

    pub fn get_img(&self, id: PropID) -> Image {
        let img = self.imgs.get_img(&id);
        let scale = *self.scaling.get(&id).unwrap();

        Image {
            img,
            scale,
        }
    }
}

pub struct Graphics {
    pub snake_graphics: SnakeGraphics,
    pub prop_graphics: PropGraphics,
}

impl Graphics {
    pub fn new() -> Self {
        Graphics {
            snake_graphics: SnakeGraphics::new(),
            prop_graphics: PropGraphics::new(),
        }
    }
}


