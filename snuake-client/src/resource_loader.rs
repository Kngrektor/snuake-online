use stdweb::traits::*;
use stdweb::web::IEventTarget;
use stdweb::web::html_element::ImageElement;

use std::hash::Hash;

extern crate serde;
use serde_json;

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;

use crate::img_placeholder::IMG_PLACEHOLDER;
use snuake_shared::*;

const USER_AVATARS: &[&str] = &[
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f438.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f43c.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f42f.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f436.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f43b.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f431.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f42a.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f439.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f42e.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f434.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f437.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f42d.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f43a.png",
    "http://raw.githubusercontent.com/EmojiTwo/emojitwo/master/png/128/1f430.png",
];

pub struct ImageLoader<T> {
    imgs: HashMap<T, ImageElement>,
}

pub type AvatarMap = ImageLoader<SnakeID>;

impl<T: Eq + Hash> ImageLoader<T>  {
    pub fn new<'a, I>(mut uris: I)  -> ImageLoader<T>
    where I: Iterator<Item = (T, &'a str)>
    {
        let imgs = uris.map(|(key, uri)| {
                let mut img = ImageElement::new();
                img.set_src(uri);
                (key, img)
            })
            .collect();

        ImageLoader { imgs }
    }

    pub fn get_img(&self, key: &T) -> ImageElement {
        if let Some(img) = self.imgs.get(key) {
            if img.complete() { return img.clone() }
        }
        let img = ImageElement::new();
        img.set_src(IMG_PLACEHOLDER);
        img
    }

    pub fn user_avatars() -> AvatarMap {
        let it = USER_AVATARS.iter()
            .enumerate()
            .map(|(i, uri)| (i as SnakeID, *uri));

        ImageLoader::new(it)
    }
}

