use rustc_serialize::base64::{CharacterSet, Config, Newline};

pub mod auth11;
pub mod auth13;

pub static BASE64_AUTH: Config = Config {
    char_set: CharacterSet::Standard,
    newline: Newline::LF,
    pad: true,
    line_length: Some(60),
};
