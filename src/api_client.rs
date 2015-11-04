use authentication::Authentication;
use config::Config;
use http_headers::*;
use hyper::Client;
use hyper::client::Response;
use hyper::client::Request as HyperRequest;
use hyper::header::{Accept, ContentType, Headers, qitem, UserAgent};
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use hyper::Result as HyperResult;
use hyper::Url;

#[doc(no_inline)]
pub use serde_json::error::Error as SerdeError;

#[doc(no_inline)]
pub use hyper::error::Error as HyperError;


#[derive(Debug)]
pub struct ApiClient {
    config: Config,
}

pub enum Error {
    Json(SerdeError),
    Hyper(HyperError),
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

    pub fn run<T>(self, req: T) -> HyperResult<Response>
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
        let mut req = try!(client.start());

        req.send()
    }

}
