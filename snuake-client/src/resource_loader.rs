use stdweb::web::html_element::ImageElement;

use std::hash::Hash;
use std::collections::HashMap;

use crate::img_placeholder::IMG_PLACEHOLDER;
use snuake_shared::*;

const USER_AVATARS: &[&str] = &[
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

pub struct ImageLoader<T> {
    imgs: HashMap<T, ImageElement>,
}

pub type AvatarMap = ImageLoader<SnakeID>;

impl<T: Eq + Hash> ImageLoader<T>  {
    pub fn new<'a, I>(uris: I)  -> ImageLoader<T>
    where I: Iterator<Item = (T, &'a str)>
    {
        let imgs = uris.map(|(key, uri)| {
                let img = ImageElement::new();
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

