use chrono::*;
use http_headers::*;
use hyper::header::Headers;
use openssl::crypto::hash::hash;
use openssl::crypto::hash::Type::SHA256;
use openssl::crypto::pkey::PKey;
use openssl::crypto::rsa::RSA;
use rustc_serialize::base64::ToBase64;
use std::fmt;
use std::fs::File;
use std::io::Read;
use utils::{squeeze_path, expand_string};
use authentication::BASE64_AUTH;
use errors::*;

#[derive(Clone)]
pub struct Auth13 {
    api_version: String,
    body: Option<String>,
    date: String,
    pub headers: Headers,
    keypath: String,
    method: String,
    path: String,
    userid: String,
    version: String,
}

impl fmt::Debug for Auth13 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Auth13")
            .field("method", &self.method)
            .field("userid", &self.userid)
            .field("path", &self.path)
            .field("body", &self.body)
            .field("keypath", &self.keypath)
            .finish()
    }
}

impl Auth13 {
    pub fn new(path: &str, key: &str, method: &str, userid: &str, api_version: &str, body: Option<String>) -> Auth13
    {
        let dt = UTC::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let userid: String = userid.into();

        let mut headers = Headers::new();
        headers.set(OpsSign(String::from("algorithm=sha256;version=1.3")));
        headers.set(OpsTimestamp(dt.clone()));
        headers.set(OpsUserId(userid.clone()));

        Auth13 {
            api_version: api_version.into(),
            body: body,
            date: dt,
            headers: headers,
            keypath: key.into(),
            method: method.into(),
            path: squeeze_path(path.into()),
            userid: userid,
            version: String::from("1.3"),
        }
    }

    fn content_hash(&self) -> Result<String> {
        let body = expand_string(&self.body);
        let content = try!(hash(SHA256, body.as_bytes())).to_base64(BASE64_AUTH);
        debug!("{:?}", content);
        Ok(content)
    }

    fn set_content_hash(mut self) -> Result<Auth13> {
        let hsh = try!(self.content_hash());
        self.headers.set(OpsContentHash(hsh));
        Ok(self)
    }

    fn canonical_user_id(&self) -> Result<String> {
        let hash = try!(hash(SHA256, &self.userid.as_bytes())).to_base64(BASE64_AUTH);
        Ok(hash)
    }

    fn canonical_request(&self) -> Result<String> {
        let cr = format!("Method:{}\nPath:{}\nX-Ops-Content-Hash:{}\n\
                         X-Ops-Sign:version=1.3\nX-Ops-Timestamp:{}\n\
                         X-Ops-UserId:{}\nX-Ops-Server-API-Version:{}",
                         &self.method,
                         &self.path,
                         try!(self.content_hash()),
                         self.date,
                         &self.userid,
                         &self.api_version);
        debug!("Canonical Request is: {:?}", cr);
        Ok(cr)
    }

    fn signed_request(&self) -> Result<String> {
        let mut key: Vec<u8> = vec![];

        match File::open(&self.keypath) {
            Ok(mut fh) => {
                try!(fh.read(&mut key));
                let cr = try!(self.canonical_request());
                let cr = cr.as_bytes();
                let key = try!(RSA::private_key_from_pem(&key));
                let key = try!(key.sign(SHA256, &cr));
                Ok(key.to_base64(BASE64_AUTH))
            }
            Err(_) => Err(ErrorKind::PrivateKeyError(self.keypath.clone()).into()),
        }
    }

    pub fn headers(self) -> Headers {
        let fin = self.set_content_hash().unwrap();
        let enc = fin.signed_request().unwrap();
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
    use super::Auth13;

    use http_headers::*;
    use hyper::header::Headers;

    const PATH: &'static str = "/organizations/clownco";
    const BODY: &'static str = "Spec Body";
    const USER: &'static str = "spec-user";
    const DT: &'static str = "2009-01-01T12:00:00Z";

    const PRIVATE_KEY: &'static str = "fixtures/spec-user.pem";

    #[test]
    fn test_new_authentication() {
        let auth = Auth13::new(PATH, PRIVATE_KEY, "GET", USER);
        assert_eq!(auth.body, None)
    }

    #[test]
    fn test_userid() {
        let auth = Auth13::new(PATH, PRIVATE_KEY, "GET", USER);
        assert_eq!(auth.userid, "spec-user");
        assert_eq!(auth.headers.get::<OpsUserId>().unwrap().to_string(),
        "spec-user")
    }

    #[test]
    fn test_method() {
        let auth = Auth13::new(PATH, PRIVATE_KEY, "GET", USER);
        assert_eq!(auth.method, "GET")
    }

    #[test]
    fn test_canonical_user_id_v13() {
        let auth = Auth13::new(PATH, PRIVATE_KEY, "GET", USER);
        assert_eq!(auth.canonical_user_id(), "EAF7Wv/hbAudWV5ZkwKz40Z/lO0=")
    }

    #[test]
    fn test_canonical_request() {
        let auth = Auth13 {
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            keypath: String::from(""),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
            version: String::from("1.3"),
        };
        assert_eq!(auth.canonical_request(),
        "Method:POST\nHashed \
        Path:YtBWDn1blGGuFIuKksdwXzHU9oE=\nX-Ops-Content-Hash:\
                    DFteJZPVv6WKdQmMqZUQUumUyRs=\nX-Ops-Timestamp:2009-01-01T12:00:\
                    00Z\nX-Ops-UserId:EAF7Wv/hbAudWV5ZkwKz40Z/lO0=")
    }

    #[test]
    fn test_private_key() {
        let auth = Auth13 {
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            keypath: String::from(PRIVATE_KEY),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
            version: String::from("1.3"),
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
        let auth = Auth13::new(PATH, PRIVATE_KEY, "GET", USER);
        let headers = auth.as_headers();

        let _ = headers.get_raw("x-ops-authorization-1").unwrap();
    }
}
