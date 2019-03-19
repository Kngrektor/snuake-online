use stdweb::traits::*;
use stdweb::web::IEventTarget;
use stdweb::web::html_element::ImageElement;

extern crate serde;
use serde_json;

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;

use snuake_shared::*;

const IMAGES: &[&str] = &[
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f438.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f43c.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f42f.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f436.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f43b.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f431.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f417.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f42a.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f439.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f42e.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f434.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f437.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f42d.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f43a.png",
    "http://github.com/EmojiTwo/emojitwo/blob/master/png/128/1f430.png",
];

pub struct ResourceLoader {
    images: Rc<RefCell<HashMap<UserID, ImageElement>>>,
}
