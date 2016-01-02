use authentication::Authentication;
use config::Config;
use http_headers::*;
use hyper::client::Response as HyperResponse;
use hyper::Client as HyperClient;
use hyper::header::{Accept, ContentType, qitem};
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::client::IntoUrl;
use std::io::Error as IoError;
use std::io::Read;
use serde_json;
use serde::ser::Serialize;
use serde::de::Deserialize;

#[doc(no_inline)]
pub use serde_json::error::Error as SerdeError;

#[doc(no_inline)]
pub use hyper::error::Error as HyperError;

pub use url::ParseError as UrlError;

#[derive(Debug,Clone)]
pub struct ApiClient {
    pub config: Config,
}

#[derive(Debug)]
pub enum Error {
    UnsuccessfulResponse(Response),
    Json(SerdeError),
    Hyper(HyperError),
    Url(UrlError),
    Generic,
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

impl From<UrlError> for Error {
    fn from(u: UrlError) -> Error {
        Error::Url(u)
    }
}

#[derive(Debug)]
pub struct Response {
    pub hyper_response: HyperResponse,
    pub body: String,
}

impl Response {
    fn from_hyper_response(mut hyper_response: HyperResponse) -> Result<Response, IoError> {
        let mut body = String::new();
        hyper_response.read_to_string(&mut body).map(|_| {
            Response {
                hyper_response: hyper_response,
                body: body,
            }
        })
    }

    pub fn from_json<T: Deserialize>(&self) -> Result<T, Error> {
        serde_json::from_str(&*self.body).map_err(|e| Error::Json(e))
    }
}

impl ApiClient {

    pub fn new(config: Config) -> ApiClient {
        ApiClient { config: config }
    }

    pub fn from_json_config(pth: &str) -> ApiClient {
        let cfg = Config::from_json(pth);
        ApiClient::new(cfg)
    }

    pub fn config(mut self, config: Config) -> ApiClient {
        self.config = config;
        self
    }

    pub fn get(&self, path: &str) -> Result<Response, Error> {
        self.send_with_body(path, "", "get")
    }

    pub fn put<B>(&self, path: &str, body: B) -> Result<Response, Error>
        where B: Serialize
    {
        let body = try!(serde_json::to_string(&body).map_err(|e| Error::Json(e)));
        self.send_with_body(path, body.as_ref(), "put")
    }

    fn send_with_body(&self, path: &str, body: &str, method: &str) -> Result<Response, Error> {
        let userid = &self.config.user.clone().unwrap();

        let auth = Authentication::new();
        let auth = auth.path(path);
        let auth = auth.key(self.config.keypath.clone().unwrap().as_ref());
        let auth = auth.method(method);
        let auth = auth.userid(userid.as_ref());

        let url = try!(format!("{}{}", &self.config.url_base(), path).into_url());

        let auth = auth.body(body.as_ref());

        let mth = match method {
            "put" => Method::Put,
            "post" => Method::Post,
            "delete" => Method::Delete,
            "head" => Method::Head,
            _ => Method::Get,
        };

        let client = HyperClient::new();
        let client = client.request(mth, url);
        let client = client.body(body.as_bytes());

        let headers = auth.as_headers();
        let mut headers = headers.clone();

        let json = Mime(TopLevel::Application, SubLevel::Json, vec![]);
        headers.set(Accept(vec![qitem(json.clone())]));
        headers.set(ContentType(json));
        headers.set(OpsApiInfo(1));
        headers.set(ChefVersion(String::from("12.5.1")));

        let client = client.headers(headers);

        let resp = try!(client.send());
        let resp = try!(Response::from_hyper_response(resp));

        if resp.hyper_response.status.is_success() {
            Ok(resp)
        } else {
            Err(Error::UnsuccessfulResponse(resp))
        }
    }

}
