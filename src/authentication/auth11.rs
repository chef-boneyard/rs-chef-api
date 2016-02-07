use chrono::*;
use http_headers::*;
use hyper::header::Headers;
use openssl::crypto::hash::hash;
use openssl::crypto::hash::Type::SHA1;
use openssl::crypto::pkey::PKey;
use rustc_serialize::base64::ToBase64;
use std::fmt;
use std::fs::File;
use utils::{squeeze_path, expand_string};
use authentication::BASE64_AUTH;

#[derive(Clone)]
pub struct Auth11 {
    body: Option<String>,
    date: String,
    pub headers: Headers,
    keypath: String,
    method: String,
    path: String,
    userid: String,
    version: String,
}

impl fmt::Debug for Auth11 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Auth11")
         .field("method", &self.method)
         .field("userid", &self.userid)
         .field("path", &self.path)
         .field("body", &self.body)
         .field("keypath", &self.keypath)
         .finish()
    }
}

impl Auth11 {
    pub fn new<P, K, M, U>(path: P, key: K, method: M, userid: U) -> Auth11
        where P: Into<String>,
              K: Into<String>,
              M: Into<String>,
              U: Into<String>
    {
        let dt = UTC::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let userid = userid.into();

        let mut headers = Headers::new();
        headers.set(OpsSign(String::from("algorithm=sha1;version=1.1")));
        headers.set(OpsTimestamp(dt.clone()));
        headers.set(OpsUserId(userid.clone()));

        Auth11 {
            body: None,
            date: dt,
            headers: headers,
            keypath: key.into(),
            method: method.into(),
            path: squeeze_path(path.into()),
            userid: userid,
            version: String::from("1.1"),
        }
    }

    pub fn body<S>(mut self, body: S) -> Auth11
        where S: Into<String>
    {
        self.body = Some(body.into());
        self
    }

    fn hashed_path(&self) -> String {
        debug!("Path is: {:?}", &self.path);
        hash(SHA1, &self.path.as_bytes()).to_base64(BASE64_AUTH)
    }

    fn content_hash(&self) -> String {
        let body = expand_string(&self.body);
        let content = hash(SHA1, body.as_bytes()).to_base64(BASE64_AUTH);
        debug!("{:?}", content);
        content
    }

    fn set_content_hash(mut self) -> Auth11 {
        let hsh = self.content_hash();
        self.headers.set(OpsContentHash(hsh));
        self
    }

    fn canonical_user_id(&self) -> String {
        hash(SHA1, &self.userid.as_bytes()).to_base64(BASE64_AUTH)
    }

    fn canonical_request(&self) -> String {
        let cr = format!("Method:{}\nHashed \
                          Path:{}\nX-Ops-Content-Hash:{}\nX-Ops-Timestamp:{}\nX-Ops-UserId:{}",
                         &self.method,
                         self.hashed_path(),
                         self.content_hash(),
                         self.date,
                         self.canonical_user_id());
        debug!("Canonical Request is: {:?}", cr);
        cr

    }

    fn encrypted_request(&self) -> String {
        match File::open(&self.keypath) {
            Ok(mut fh) => {
                let key = PKey::private_key_from_pem(&mut fh).unwrap();
                key.private_encrypt(&self.canonical_request().as_bytes()).to_base64(BASE64_AUTH)
            }
            Err(_) => panic!("Couldn't open private key"),
        }
    }

    pub fn as_headers(self) -> Headers {
        let fin = self.set_content_hash();
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
    use super::Auth11;

    use http_headers::*;
    use hyper::header::Headers;

    const PATH: &'static str = "/organizations/clownco";
    const BODY: &'static str = "Spec Body";
    const USER: &'static str = "spec-user";
    const DT: &'static str = "2009-01-01T12:00:00Z";

    const PRIVATE_KEY: &'static str = "fixtures/spec-user.pem";

    #[test]
    fn test_new_authentication() {
        let auth = Auth11::new(PATH, PRIVATE_KEY, "GET", USER);
        assert_eq!(auth.body, None)
    }

    #[test]
    fn test_userid() {
        let auth = Auth11::new(PATH, PRIVATE_KEY, "GET", USER);
        assert_eq!(auth.userid, "spec-user");
        assert_eq!(auth.headers.get::<OpsUserId>().unwrap().to_string(),
                   "spec-user")
    }

    #[test]
    fn test_method() {
        let auth = Auth11::new(PATH, PRIVATE_KEY, "GET", USER);
        assert_eq!(auth.method, "GET")
    }

    #[test]
    fn test_canonical_user_id_v11() {
        let auth = Auth11::new(PATH, PRIVATE_KEY, "GET", USER);
        assert_eq!(auth.canonical_user_id(), "EAF7Wv/hbAudWV5ZkwKz40Z/lO0=")
    }

    #[test]
    fn test_canonical_request() {
        let auth = Auth11 {
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            keypath: String::from(""),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
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
        let auth = Auth11 {
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            keypath: String::from(PRIVATE_KEY),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
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
        let auth = Auth11::new(PATH, PRIVATE_KEY, "GET", USER);
        let headers = auth.as_headers();

        let _ = headers.get_raw("x-ops-authorization-1").unwrap();
    }
}
