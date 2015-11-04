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
use utils::squeeze_path;

pub struct Authentication {
    body: Option<String>,
    date: String,
    headers: Headers,
    key: Option<PKey>,
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
            key: None,
            method: None,
            path: None,
            userid: None,
            version: String::from("1.1"),
        }
    }

    pub fn body(mut self, body: &str) -> Authentication {
        let body = String::from(body);
        self.body = Some(body);
        self
    }

    pub fn key(mut self, keypath: &str) -> Authentication {
        match File::open(keypath) {
            Ok(mut fh) => {
                self.key = Some(PKey::private_key_from_pem(&mut fh).unwrap());
                self
            }
            Err(_) => panic!("Couldn't open private key"),
        }
    }

    pub fn method(mut self, method: &str) -> Authentication {
        let method = String::from(method.to_ascii_uppercase());
        self.method = Some(method);
        self
    }

    pub fn path(mut self, path: &str) -> Authentication {
        let path = String::from(path);
        self.path = Some(squeeze_path(path));
        self
    }

    pub fn userid(mut self, id: &str) -> Authentication {
        let userid = String::from(id);
        self.userid = Some(userid.clone());
        self.headers.set(OpsUserId(userid));
        self
    }

    pub fn version(mut self, version: &str) -> Authentication {
        let version = String::from(version);
        self.version = version;
        self
    }

    pub fn set_timestamp(mut self) -> Authentication {
        self.headers.set(OpsTimestamp(self.date.clone()));
        self
    }

    fn hashed_path(&self) -> String {
        debug!("{:?}", &self.path);
        hash(SHA1, expand_string(&self.path).as_bytes()).to_base64(BASE64_AUTH)
    }

    /// FIXME: this needs, eventually, to deal with IO and not just strings
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
        debug!("{:?}", cr);
        cr

    }

    fn encrypted_request(&self) -> String {
        match self.key {
            Some(ref key) =>
                key.private_encrypt(&self.canonical_request().as_bytes()).to_base64(BASE64_AUTH),
            None => panic!("No private key provided!"),
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

fn expand_string(val: &Option<String>) -> String {
    match *val {
        None => "".to_string(),
        Some(ref x) => x.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::Authentication;

    use http_headers::*;
    use hyper::header::Headers;
    use openssl::crypto::pkey::PKey;
    use std::fs::File;

    const PATH: &'static str = "/organizations/clownco";
    const BODY: &'static str = "Spec Body";
    const USER: &'static str = "spec-user";
    const DT: &'static str = "2009-01-01T12:00:00Z";

    const PRIVATE_KEY: &'static str = "fixtures/spec-user.pem";

    const PRIVATE_KEY_DATA: &'static str = r"
-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA0ueqo76MXuP6XqZBILFziH/9AI7C6PaN5W0dSvkr9yInyGHS
z/IR1+4tqvP2qlfKVKI4CP6BFH251Ft9qMUBuAsnlAVQ1z0exDtIFFOyQCdR7iXm
jBIWMSS4buBwRQXwDK7id1OxtU23qVJv+xwEV0IzaaSJmaGLIbvRBD+qatfUuQJB
MU/04DdJIwvLtZBYdC2219m5dUBQaa4bimL+YN9EcsDzD9h9UxQo5ReK7b3cNMzJ
BKJWLzFBcJuePMzAnLFktr/RufX4wpXe6XJxoVPaHo72GorLkwnQ0HYMTY8rehT4
mDi1FI969LHCFFaFHSAaRnwdXaQkJmSfcxzCYQIDAQABAoIBAQCW3I4sKN5B9jOe
xq/pkeWBq4OvhW8Ys1yW0zFT8t6nHbB1XrwscQygd8gE9BPqj3e0iIEqtdphbPmj
VHqTYbC0FI6QDClifV7noTwTBjeIOlgZ0NSUN0/WgVzIOxUz2mZ2vBZUovKILPqG
TOi7J7RXMoySMdcXpP1f+PgvYNcnKsT72UcWaSXEV8/zo+Zm/qdGPVWwJonri5Mp
DVm5EQSENBiRyt028rU6ElXORNmoQpVjDVqZ1gipzXkifdjGyENw2rt4V/iKYD7V
5iqXOsvP6Cemf4gbrjunAgDG08S00kiUgvVWcdXW+dlsR2nCvH4DOEe3AYYh/aH8
DxEE7FbtAoGBAPcNO8fJ56mNw0ow4Qg38C+Zss/afhBOCfX4O/SZKv/roRn5+gRM
KRJYSVXNnsjPI1plzqR4OCyOrjAhtuvL4a0DinDzf1+fiztyNohwYsW1vYmqn3ti
EN0GhSgE7ppZjqvLQ3f3LUTxynhA0U+k9wflb4irIlViTUlCsOPkrNJDAoGBANqL
Q+vvuGSsmRLU/Cenjy+Mjj6+QENg51dz34o8JKuVKIPKU8pNnyeLa5fat0qD2MHm
OB9opeQOcw0dStodxr6DB3wi83bpjeU6BWUGITNiWEaZEBrQ0aiqNJJKrrHm8fAZ
9o4l4oHc4hI0kYVYYDuxtKuVJrzZiEapTwoOcYiLAoGBAI/EWbeIHZIj9zOjgjEA
LHvm25HtulLOtyk2jd1njQhlHNk7CW2azIPqcLLH99EwCYi/miNH+pijZ2aHGCXb
/bZrSxM0ADmrZKDxdB6uGCyp+GS2sBxjEyEsfCyvwhJ8b3Q100tqwiNO+d5FCglp
HICx2dgUjuRVUliBwOK93nx1AoGAUI8RhIEjOYkeDAESyhNMBr0LGjnLOosX+/as
qiotYkpjWuFULbibOFp+WMW41vDvD9qrSXir3fstkeIAW5KqVkO6mJnRoT3Knnra
zjiKOITCAZQeiaP8BO5o3pxE9TMqb9VCO3ffnPstIoTaN4syPg7tiGo8k1SklVeH
2S8lzq0CgYAKG2fljIYWQvGH628rp4ZcXS4hWmYohOxsnl1YrszbJ+hzR+IQOhGl
YlkUQYXhy9JixmUUKtH+NXkKX7Lyc8XYw5ETr7JBT3ifs+G7HruDjVG78EJVojbd
8uLA+DdQm5mg4vd1GTiSK65q/3EeoBlUaVor3HhLFki+i9qpT8CBsg==
-----END RSA PRIVATE KEY-----
";

    // const PUBLIC_KEY_DATA: &'static str = r"
    // -----BEGIN PUBLIC KEY-----
    // MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0ueqo76MXuP6XqZBILFz
    // iH/9AI7C6PaN5W0dSvkr9yInyGHSz/IR1+4tqvP2qlfKVKI4CP6BFH251Ft9qMUB
    // uAsnlAVQ1z0exDtIFFOyQCdR7iXmjBIWMSS4buBwRQXwDK7id1OxtU23qVJv+xwE
    // V0IzaaSJmaGLIbvRBD+qatfUuQJBMU/04DdJIwvLtZBYdC2219m5dUBQaa4bimL+
    // YN9EcsDzD9h9UxQo5ReK7b3cNMzJBKJWLzFBcJuePMzAnLFktr/RufX4wpXe6XJx
    // oVPaHo72GorLkwnQ0HYMTY8rehT4mDi1FI969LHCFFaFHSAaRnwdXaQkJmSfcxzC
    // YQIDAQAB
    // -----END PUBLIC KEY-----
    // ";

    #[test]
    fn test_new_authentication() {
        let auth = Authentication::new();
        assert_eq!(auth.body, None)
    }

    #[test]
    fn test_load_key() {
        let mut fh = File::open(PRIVATE_KEY).unwrap();
        let k0 = PKey::private_key_from_pem(&mut fh).unwrap();
        let auth = Authentication::new().key(PRIVATE_KEY);
        assert!(auth.key.unwrap().public_eq(&k0))
    }

    #[test]
    fn test_timestamp() {
        let auth = Authentication {
                       body: None,
                       date: String::from(DT),
                       headers: Headers::new(),
                       key: None,
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
            key: None,
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
        let k0 = PKey::private_key_from_pem(&mut PRIVATE_KEY_DATA.as_bytes()).unwrap();

        let auth = Authentication {
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            key: Some(k0),
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
        let k0 = PKey::private_key_from_pem(&mut PRIVATE_KEY_DATA.as_bytes()).unwrap();

        let auth = Authentication {
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            key: Some(k0),
            method: Some(String::from("POST")),
            path: Some(String::from(PATH)),
            userid: Some(String::from(USER)),
            version: String::from("1.1"),
        };
        let headers = auth.as_headers();

        let header = headers.get_raw("x-ops-authorization-1").unwrap();
    }



}
