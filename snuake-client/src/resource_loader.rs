use stdweb::web::html_element::ImageElement;

use std::hash::Hash;
use std::collections::HashMap;

use crate::img_placeholder::IMG_PLACEHOLDER;

pub struct ImageLoader<T> {
    imgs: HashMap<T, ImageElement>,
}

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
}

