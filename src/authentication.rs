use chrono::*;
use http_headers::*;
use hyper::header::Headers;
use openssl::crypto::hash::hash;
use openssl::crypto::hash::Type::SHA1;
use openssl::crypto::pkey::PKey;
use rustc_serialize::base64::{ToBase64, Config, Newline, CharacterSet};
use std::ascii::AsciiExt;
use std::fmt;
use std::fs::File;
use utils::{squeeze_path, expand_string};

#[derive(Clone)]
pub struct Authentication {
    body: Option<String>,
    date: String,
    headers: Headers,
    keypath: Option<String>,
    method: Option<String>,
    path: Option<String>,
    userid: Option<String>,
    version: String,
}

pub static BASE64_AUTH: Config = Config {
    char_set: CharacterSet::Standard,
    newline: Newline::LF,
    pad: true,
    line_length: Some(60),
};

impl fmt::Debug for Authentication {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Authentication")
         .field("method", &self.method)
         .field("userid", &self.userid)
         .field("path", &self.path)
         .field("body", &self.body)
         .field("keypath", &self.keypath)
         .finish()
    }
}

impl Authentication {
    pub fn new() -> Authentication {
        let dt = UTC::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let mut headers = Headers::new();
        headers.set(OpsSign(String::from("algorithm=sha1;version=1.1")));
        Authentication {
            body: None,
            date: dt,
            headers: headers,
            keypath: None,
            method: None,
            path: None,
            userid: None,
            version: String::from("1.1"),
        }
    }

    pub fn body<S>(mut self, body: S) -> Authentication
        where S: Into<String>
    {
        self.body = Some(body.into());
        self
    }

    pub fn key<S>(mut self, keypath: S) -> Authentication
        where S: Into<String>
    {
        self.keypath = Some(keypath.into());
        self
    }

    pub fn method<S>(mut self, method: S) -> Authentication
        where S: Into<String>
    {
        let method = method.into().to_ascii_uppercase();
        self.method = Some(method);
        self
    }

    pub fn path<S>(mut self, path: S) -> Authentication
        where S: Into<String>
    {
        self.path = Some(squeeze_path(path.into()));
        self
    }

    pub fn userid<S>(mut self, id: S) -> Authentication
        where S: Into<String>
    {
        let userid = id.into();
        self.userid = Some(userid.clone());
        self.headers.set(OpsUserId(userid));
        self
    }

    pub fn version<S>(mut self, version: S) -> Authentication
        where S: Into<String>
    {
        self.version = version.into();
        self
    }

    pub fn set_timestamp(mut self) -> Authentication {
        self.headers.set(OpsTimestamp(self.date.clone()));
        self
    }

    fn hashed_path(&self) -> String {
        debug!("Path is: {:?}", &self.path);
        hash(SHA1, expand_string(&self.path).as_bytes()).to_base64(BASE64_AUTH)
    }

    fn content_hash(&self) -> String {
        let body = expand_string(&self.body);
        let content = hash(SHA1, body.as_bytes()).to_base64(BASE64_AUTH);
        debug!("{:?}", content);
        content
    }

    fn set_content_hash(mut self) -> Authentication {
        let hsh = self.content_hash();
        self.headers.set(OpsContentHash(hsh));
        self
    }

    fn canonical_user_id(&self) -> String {
        let userid = expand_string(&self.userid);
        if self.version == "1.0" {
            userid
        } else {
            hash(SHA1, userid.as_bytes()).to_base64(BASE64_AUTH)
        }
    }

    fn canonical_request(&self) -> String {
        let cr = format!("Method:{}\nHashed \
                          Path:{}\nX-Ops-Content-Hash:{}\nX-Ops-Timestamp:{}\nX-Ops-UserId:{}",
                         expand_string(&self.method),
                         self.hashed_path(),
                         self.content_hash(),
                         self.date,
                         self.canonical_user_id());
        debug!("Canonical Request is: {:?}", cr);
        cr

    }

    fn encrypted_request(&self) -> String {
        if let Some(ref key) = self.keypath {
            match File::open(key) {
                Ok(mut fh) => {
                    let key = PKey::private_key_from_pem(&mut fh).unwrap();
                    key.private_encrypt(&self.canonical_request().as_bytes()).to_base64(BASE64_AUTH)
                }
                Err(_) => panic!("Couldn't open private key"),
            }
        } else {
            panic!("No key path provided!")
        }
    }

    pub fn as_headers(self) -> Headers {
        let fin = self.set_timestamp();
        let fin = fin.set_content_hash();
        let enc = fin.encrypted_request();
        let mut headers = fin.headers;
        let mut i = 1;
        for h in enc.split('\n') {
            let key = format!("X-Ops-Authorization-{}", i);
            headers.set_raw(key, vec![h.as_bytes().to_vec()]);
            i += 1;
        }
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::Authentication;

    use http_headers::*;
    use hyper::header::Headers;

    const PATH: &'static str = "/organizations/clownco";
    const BODY: &'static str = "Spec Body";
    const USER: &'static str = "spec-user";
    const DT: &'static str = "2009-01-01T12:00:00Z";

    const PRIVATE_KEY: &'static str = "fixtures/spec-user.pem";

    #[test]
    fn test_new_authentication() {
        let auth = Authentication::new();
        assert_eq!(auth.body, None)
    }

    #[test]
    fn test_timestamp() {
        let auth = Authentication {
                       body: None,
                       date: String::from(DT),
                       headers: Headers::new(),
                       keypath: None,
                       method: None,
                       path: None,
                       userid: None,
                       version: String::from("1.1"),
                   }
                   .set_timestamp();
        assert_eq!(auth.headers.get::<OpsTimestamp>().unwrap().to_string(),
                   "2009-01-01T12:00:00Z")
    }

    #[test]
    fn test_userid() {
        let auth = Authentication::new().userid(USER);
        assert_eq!(auth.userid.unwrap(), "spec-user");
        assert_eq!(auth.headers.get::<OpsUserId>().unwrap().to_string(),
                   "spec-user")
    }

    #[test]
    fn test_method() {
        let auth = Authentication::new().method("get");
        assert_eq!(auth.method.unwrap(), "GET")
    }

    #[test]
    fn test_canonical_user_id_v1_0() {
        let auth = Authentication::new().userid(USER).version("1.0");
        assert_eq!(auth.canonical_user_id(), "spec-user")
    }

    #[test]
    fn test_canonical_user_id_v1_1() {
        let auth = Authentication::new().userid(USER);
        assert_eq!(auth.canonical_user_id(), "EAF7Wv/hbAudWV5ZkwKz40Z/lO0=")
    }

    #[test]
    fn test_canonical_request() {
        let auth = Authentication {
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            keypath: None,
            method: Some(String::from("POST")),
            path: Some(String::from(PATH)),
            userid: Some(String::from(USER)),
            version: String::from("1.1"),
        };
        assert_eq!(auth.canonical_request(),
                   "Method:POST\nHashed \
                    Path:YtBWDn1blGGuFIuKksdwXzHU9oE=\nX-Ops-Content-Hash:\
                    DFteJZPVv6WKdQmMqZUQUumUyRs=\nX-Ops-Timestamp:2009-01-01T12:00:\
                    00Z\nX-Ops-UserId:EAF7Wv/hbAudWV5ZkwKz40Z/lO0=")
    }

    #[test]
    fn test_private_key() {
        let auth = Authentication {
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            keypath: Some(String::from(PRIVATE_KEY)),
            method: Some(String::from("POST")),
            path: Some(String::from(PATH)),
            userid: Some(String::from(USER)),
            version: String::from("1.1"),
        };
        assert_eq!(&auth.encrypted_request(),
                   "UfZD9dRz6rFu6LbP5Mo1oNHcWYxpNIcUfFCffJS1FQa0GtfU/vkt3/O5HuCM\n1wIFl/U0f5faH9EW\
                    pXWY5NwKR031Myxcabw4t4ZLO69CIh/3qx1XnjcZvt2w\nc2R9bx/43IWA/r8w8Q6decuu0f6ZlNhe\
                    JeJhaYPI8piX/aH+uHBH8zTACZu8\nvMnl5MF3/OIlsZc8cemq6eKYstp8a8KYq9OmkB5IXIX6qVMJ\
                    HA6fRvQEB/7j\n281Q7oI/O+lE8AmVyBbwruPb7Mp6s4839eYiOdjbDwFjYtbS3XgAjrHlaD7W\nFD\
                    lbAG7H8Dmvo+wBxmtNkszhzbBnEYtuwQqT8nM/8A==")
    }

    #[test]
    fn test_headers() {
        let auth = Authentication {
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            keypath: Some(String::from(PRIVATE_KEY)),
            method: Some(String::from("POST")),
            path: Some(String::from(PATH)),
            userid: Some(String::from(USER)),
            version: String::from("1.1"),
        };
        let headers = auth.as_headers();

        let _ = headers.get_raw("x-ops-authorization-1").unwrap();
    }



}
