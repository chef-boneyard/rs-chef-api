use chrono::*;
use http_headers::*;
use hyper::header::Headers;
use openssl::hash::{MessageDigest, hash2};
use openssl::sign::Signer;
use openssl::pkey::PKey;
use rustc_serialize::base64::ToBase64;
use std::fmt;
use std::fs::File;
use std::io::Read;
use utils::{expand_string, squeeze_path};
use authentication::BASE64_AUTH;
use failure::Error;
use errors::ChefError;

pub struct Auth13 {
    api_version: String,
    body: Option<String>,
    date: String,
    keypath: String,
    method: String,
    path: String,
    userid: String,
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
    pub fn new(
        path: &str,
        key: &str,
        method: &str,
        userid: &str,
        api_version: &str,
        body: Option<String>,
    ) -> Auth13 {
        let dt = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let userid: String = userid.into();
        let method = String::from(method).to_ascii_uppercase();

        Auth13 {
            api_version: api_version.into(),
            body: body,
            date: dt,
            keypath: key.into(),
            method: method.into(),
            path: squeeze_path(path.into()),
            userid: userid,
        }
    }

    fn content_hash(&self) -> Result<String, Error> {
        let body = expand_string(&self.body);
        let content = hash2(MessageDigest::sha256(), body.as_bytes())?.to_base64(BASE64_AUTH);
        debug!("{:?}", content);
        Ok(content)
    }

    fn canonical_request(&self) -> Result<String, Error> {
        let cr = format!(
            "Method:{}\nPath:{}\nX-Ops-Content-Hash:{}\n\
             X-Ops-Sign:version=1.3\nX-Ops-Timestamp:{}\n\
             X-Ops-UserId:{}\nX-Ops-Server-API-Version:{}",
            &self.method,
            &self.path,
            try!(self.content_hash()),
            self.date,
            &self.userid,
            &self.api_version
        );
        debug!("Canonical Request is: {:?}", cr);
        Ok(cr)
    }

    fn signed_request(&self) -> Result<String, Error> {
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
            Err(_) => Err(ChefError::PrivateKeyError(self.keypath.clone()).into()),
        }
    }

    pub fn build(self, headers: &mut Headers) -> Result<(), Error> {
        let hsh = self.content_hash()?;
        headers.set(OpsContentHash(hsh));
        headers.set(OpsSign(String::from("algorithm=sha256;version=1.3")));
        headers.set(OpsTimestamp(self.date.clone()));
        headers.set(OpsUserId(self.userid.clone()));

        let enc = try!(self.signed_request());
        let mut i = 1;
        for h in enc.split('\n') {
            let key = format!("X-Ops-Authorization-{}", i);
            headers.set_raw(key, vec![h.as_bytes().to_vec()]);
            i += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Auth13;

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
    fn test_canonical_request() {
        let auth = Auth13 {
            api_version: String::from("1"),
            body: Some(String::from(BODY)),
            date: String::from(DT),
            keypath: String::from(PRIVATE_KEY),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
        };
        assert_eq!(
            auth.canonical_request().unwrap(),
            "Method:POST\nPath:/organizations/clownco\nX-Ops-Content-Hash:\
             hDlKNZhIhgso3Fs0S0pZwJ0xyBWtR1RBaeHs1DrzOho=\nX-Ops-Sign:version=1.\
             3\nX-Ops-Timestamp:2009-01-01T12:00:00Z\nX-Ops-UserId:\
             spec-user\nX-Ops-Server-API-Version:1"
        )
    }

    #[test]
    fn test_signed_request() {
        let auth = Auth13 {
            api_version: String::from("1"),
            body: Some(String::from(BODY)),
            date: String::from(DT),
            keypath: String::from(PRIVATE_KEY),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
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

        assert_eq!(
            sig,
            "FZOmXAyOBAZQV/uw188iBljBJXOm+m8xQ/8KTGLkgGwZNcRFxk1m953XjE3W\n\
             VGy1dFT76KeaNWmPCNtDmprfH2na5UZFtfLIKrPv7xm80V+lzEzTd9WBwsfP\n\
             42dZ9N+V9I5SVfcL/lWrrlpdybfceJC5jOcP5tzfJXWUITwb6Z3Erg3DU3Uh\n\
             H9h9E0qWlYGqmiNCVrBnpe6Si1gU/Jl+rXlRSNbLJ4GlArAPuL976iTYJTzE\n\
             MmbLUIm3JRYi00Yb01IUCCKdI90vUq1HHNtlTEu93YZfQaJwRxXlGkCNwIJe\n\
             fy49QzaCIEu1XiOx5Jn+4GmkrZch/RrK9VzQWXgs+w=="
        )
    }

}
