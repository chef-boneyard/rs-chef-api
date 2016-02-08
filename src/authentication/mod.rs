use rustc_serialize::base64::{Config, Newline, CharacterSet};
use hyper::header::Headers;
use utils::squeeze_path;
use std::ascii::AsciiExt;

pub mod auth11;
use authentication::auth11::Auth11;

pub static BASE64_AUTH: Config = Config {
    char_set: CharacterSet::Standard,
    newline: Newline::LF,
    pad: true,
    line_length: Some(60),
};

#[derive(Clone)]
pub struct Authentication {
    body: Option<String>,
    keypath: String,
    method: String,
    path: String,
    userid: String,
    version: String,
}

impl Authentication {
    pub fn new<P, K, M, U, V>(path: P, key: K, method: M, userid: U, version: V) -> Authentication
        where P: Into<String>,
              K: Into<String>,
              M: Into<String>,
              U: Into<String>,
              V: Into<String>
              {
                  Authentication {
                      body: None,
                      keypath: key.into(),
                      method: method.into().to_ascii_uppercase(),
                      path: squeeze_path(path.into()),
                      userid: userid.into(),
                      version: version.into(),
                  }

              }

    pub fn body<S>(mut self, body: S) -> Authentication
        where S: Into<String>
        {
            self.body = Some(body.into());
            self
        }

    pub fn headers(self) -> Headers {
        Auth11::new(&self.path, &self.keypath, &self.method, &self.userid, self.body).headers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    const PATH: &'static str = "/organizations/clownco";
    const BODY: &'static str = "Spec Body";
    const USER: &'static str = "spec-user";
    const DT: &'static str = "2009-01-01T12:00:00Z";

    const PRIVATE_KEY: &'static str = "fixtures/spec-user.pem";

    #[test]
    fn test_auth_return() {
        let auth = Authentication::new(PATH, PRIVATE_KEY, "POST", USER, "1.1");
        let headers = auth.body(BODY).headers();
        let first = "UfZD9dRz6rFu6LbP5Mo1oNHcWYxpNIcUfFCffJS1FQa0GtfU/vkt3/O5HuCM";
        let out = headers.get_raw("x-ops-authorization-1").unwrap();
        assert_eq!(first.to_owned(), String::from_utf8(out[0].clone()).unwrap())
    }
}
