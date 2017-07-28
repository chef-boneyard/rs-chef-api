use chrono::*;
use http_headers::*;
use hyper::header::Headers;
use openssl::hash::{MessageDigest, hash};
use openssl::sign::Signer;
use openssl::pkey::PKey;
use rustc_serialize::base64::ToBase64;
use std::fmt;
use std::fs::File;
use std::io::Read;
use utils::{squeeze_path, expand_string};
use authentication::{BASE64_AUTH, Authenticator};
use errors::*;
use std::ascii::AsciiExt;

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
    pub fn new(path: &str,
               key: &str,
               method: &str,
               userid: &str,
               api_version: &str,
               body: Option<String>)
               -> impl Authenticator {
        let dt = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let userid: String = userid.into();
        let method = String::from(method).to_ascii_uppercase();

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
        let content = try!(hash(MessageDigest::sha256(), body.as_bytes())).to_base64(BASE64_AUTH);
        debug!("{:?}", content);
        Ok(content)
    }

    fn set_content_hash(mut self) -> Result<Auth13> {
        let hsh = try!(self.content_hash());
        self.headers.set(OpsContentHash(hsh));
        Ok(self)
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
                try!(fh.read_to_end(&mut key));
                let key = try!(PKey::private_key_from_pem(key.as_slice()));

                let cr = try!(self.canonical_request());
                let cr = cr.as_bytes();

                let mut signer = try!(Signer::new(MessageDigest::sha256(), &key));
                signer.update(cr).unwrap();
                let result = try!(signer.finish());

                Ok(result.to_base64(BASE64_AUTH))
            }
            Err(_) => Err(ErrorKind::PrivateKeyError(self.keypath.clone()).into()),
        }
    }
}

impl Authenticator for Auth13 {
    fn headers(self) -> Result<Headers> {
        let fin = try!(self.set_content_hash());
        let enc = try!(fin.signed_request());
        let mut headers = fin.headers;
        let mut i = 1;
        for h in enc.split('\n') {
            let key = format!("X-Ops-Authorization-{}", i);
            headers.set_raw(key, vec![h.as_bytes().to_vec()]);
            i += 1;
        }
        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use super::Auth13;
    use authentication::Authenticator;

    use http_headers::*;
    use hyper::header::Headers;

    use openssl::hash::MessageDigest;
    use openssl::sign::Verifier;
    use openssl::pkey::PKey;
    use rustc_serialize::base64::FromBase64;
    use std::fs::File;
    use std::io::Read;

    const PATH: &'static str = "/organizations/clownco";
    const BODY: &'static str = "Spec Body";
    const USER: &'static str = "spec-user";
    const DT: &'static str = "2009-01-01T12:00:00Z";

    const PRIVATE_KEY: &'static str = "fixtures/spec-user.pem";

    #[test]
    fn test_userid() {
        let auth = Auth13::new(PATH, PRIVATE_KEY, "GET", USER, "0", None);
        assert_eq!(auth.headers().unwrap().get::<OpsUserId>().unwrap().to_string(),
                   "spec-user")
    }

    #[test]
    fn test_canonical_request() {
        let auth = Auth13 {
            api_version: String::from("1"),
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            keypath: String::from(PRIVATE_KEY),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
            version: String::from("1.3"),
        };
        assert_eq!(auth.canonical_request().unwrap(),
                   "Method:POST\nPath:/organizations/clownco\nX-Ops-Content-Hash:\
                    hDlKNZhIhgso3Fs0S0pZwJ0xyBWtR1RBaeHs1DrzOho=\nX-Ops-Sign:version=1.\
                    3\nX-Ops-Timestamp:2009-01-01T12:00:00Z\nX-Ops-UserId:\
                    spec-user\nX-Ops-Server-API-Version:1")
    }

    #[test]
    fn test_signed_request() {
        let auth = Auth13 {
            api_version: String::from("1"),
            body: Some(String::from(BODY)),
            date: String::from(DT),
            headers: Headers::new(),
            keypath: String::from(PRIVATE_KEY),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
            version: String::from("1.3"),
        };
        let sig = &auth.signed_request().unwrap();
        let req = &auth.canonical_request().unwrap();

        let sig_raw = sig.clone().from_base64().unwrap();
        let mut key: Vec<u8> = vec![];
        let mut fh = File::open(PRIVATE_KEY).unwrap();
        fh.read_to_end(&mut key).unwrap();
        let key = PKey::private_key_from_pem(key.as_slice()).unwrap();

        let mut ver = Verifier::new(MessageDigest::sha256(), &key).unwrap();
        ver.update(req.as_bytes()).unwrap();
        assert!(ver.finish(sig_raw.as_slice()).unwrap());

        assert_eq!(sig,
                   "FZOmXAyOBAZQV/uw188iBljBJXOm+m8xQ/8KTGLkgGwZNcRFxk1m953XjE3W\n\
                   VGy1dFT76KeaNWmPCNtDmprfH2na5UZFtfLIKrPv7xm80V+lzEzTd9WBwsfP\n\
                   42dZ9N+V9I5SVfcL/lWrrlpdybfceJC5jOcP5tzfJXWUITwb6Z3Erg3DU3Uh\n\
                   H9h9E0qWlYGqmiNCVrBnpe6Si1gU/Jl+rXlRSNbLJ4GlArAPuL976iTYJTzE\n\
                   MmbLUIm3JRYi00Yb01IUCCKdI90vUq1HHNtlTEu93YZfQaJwRxXlGkCNwIJe\n\
                   fy49QzaCIEu1XiOx5Jn+4GmkrZch/RrK9VzQWXgs+w==")
    }

    #[test]
    fn test_headers() {
        let auth = Auth13::new(PATH,
                               PRIVATE_KEY,
                               "GET",
                               USER,
                               "1",
                               Some(String::from(BODY)));
        let headers = auth.headers().unwrap();

        assert!(headers.get_raw("x-ops-authorization-1").is_some())
    }
}
