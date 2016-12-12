use rustc_serialize::base64::{Config, Newline, CharacterSet};
use hyper::header::Headers;
use errors::*;

pub mod auth11;
pub mod auth13;

pub static BASE64_AUTH: Config = Config {
    char_set: CharacterSet::Standard,
    newline: Newline::LF,
    pad: true,
    line_length: Some(60),
};

pub trait Authenticator {
    fn headers(self) -> Result<Headers>;
}
