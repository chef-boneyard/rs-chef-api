use authentication::Authentication;
use config::Config;
use http_headers::*;
use hyper::client::Response as HyperResponse;
use hyper::client::Request as HyperRequest;
use hyper::header::{Accept, ContentType, qitem};
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::Url;
use std::io::Error as IoError;
use std::io::Read;
use serde_json;
use serde::de::Deserialize;

#[doc(no_inline)]
pub use serde_json::error::Error as SerdeError;

#[doc(no_inline)]
pub use hyper::error::Error as HyperError;


#[derive(Debug,Clone)]
pub struct ApiClient {
    config: Config,
}

#[derive(Debug)]
pub enum Error {
    UnsuccessfulResponse(Response),
    Json(SerdeError),
    Hyper(HyperError),
}

impl From<HyperError> for Error {
    fn from(h: HyperError) -> Error {
        Error::Hyper(h)
    }
}

impl From<IoError> for Error {
    fn from(i: IoError) -> Error {
        Error::Hyper(HyperError::Io(i))
    }
}

#[derive(Debug)]
pub struct Response {
    pub hyper_response: HyperResponse,
    pub body: String
}

impl Response {
    fn from_hyper_response(mut hyper_response: HyperResponse) -> Result<Response, IoError> {
        let mut body = String::new();
        hyper_response.read_to_string(&mut body).map(|_| Response{ hyper_response: hyper_response, body: body })
    }

    pub fn from_json<T: Deserialize>(&self) -> Result<T, Error> {
        serde_json::from_str(&*self.body).map_err(|e| Error::Json(e))
    }
}

pub trait Request {
    fn full_path(&self, base: &str) -> String;
    fn method(&self) -> Method;
}

impl ApiClient {
    pub fn new(config: Config) -> ApiClient {
        ApiClient { config: config }
    }

    pub fn config(mut self, config: Config) -> ApiClient {
        self.config = config;
        self
    }

    pub fn run<T>(self, req: T) -> Result<Response, Error>
        where T: Request {
        let org = &self.config.organization_path();
        let base = &self.config.url_base();
        let method: Method = req.method();
        let userid = &self.config.user.clone().unwrap();

        let full_path = &req.full_path(org.as_ref());
        let url = Url::parse(format!("{}{}", base, full_path).as_ref()).unwrap();

        debug!("URL is {}", url);
        let auth = Authentication::new();
        let auth = auth.path(full_path.as_ref());
        let auth = auth.key(self.config.keypath.clone().unwrap().as_ref());
        let auth = auth.method(method.as_ref());
        let auth = auth.userid(userid.as_ref());

        let mut headers = auth.as_headers();

        let json = Mime(TopLevel::Application, SubLevel::Json, vec![]);
        headers.set(Accept(vec![qitem(json.clone())]));
        headers.set(ContentType(json));
        headers.set(OpsApiInfo(1));
        headers.set(ChefVersion(String::from("12.5.1")));

        let mut client = try!(HyperRequest::new(method, url));
        client.headers_mut().extend(headers.iter());
        let req = try!(client.start());

        let resp = try!(req.send());
        let resp = try!(Response::from_hyper_response(resp));

        if resp.hyper_response.status.is_success() {
            Ok(resp)
        } else {
            Err(Error::UnsuccessfulResponse(resp))
        }
    }

}
