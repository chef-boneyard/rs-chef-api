use authentication::auth11::Auth11;
use authentication::auth13::Auth13;
use config::Config;
use http_headers::*;

use hyper::Client as HyperClient;
use hyper::header::{qitem, Accept, ContentLength, ContentType};
use hyper::{Method, Request};
use hyper::mime::APPLICATION_JSON;
use hyper_openssl::HttpsConnector;

use tokio_core::reactor::Core;
use futures::{Future, Stream};

use std::io;
use serde_json;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use errors::*;

#[derive(Debug, Clone)]
pub struct ApiClient {
    pub config: Config,
}

impl ApiClient {
    pub fn new(config: Config) -> Self {
        ApiClient { config: config }
    }

    pub fn from_json_config(pth: &str) -> Result<Self> {
        Config::from_json(pth).map(ApiClient::new)
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.send_with_body(path, &String::from(""), "get")
    }

    pub fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.send_with_body(path, &String::from(""), "delete")
    }

    pub fn post<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        self.send_with_body(path, body, "post")
    }

    pub fn put<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        self.send_with_body(path, body, "put")
    }

    fn send_with_body<B, T> (
        &self,
        path: &str,
        body: &B,
        method: &str,
    ) -> Result<T> 
    where 
    B: Serialize,
    T: DeserializeOwned
    {
        let userid = self.config.user.clone().unwrap();
        let keypath = self.config.keypath.clone().unwrap();
        let sign_ver = self.config.sign_ver.clone();

        let url = try!(format!("{}{}", &self.config.url_base(), path).parse());

        let mth = match method {
            "put" => Method::Put,
            "post" => Method::Post,
            "delete" => Method::Delete,
            "head" => Method::Head,
            _ => Method::Get,
        };

        let mut request = Request::new(mth, url);

        let body = try!(serde_json::to_string(&body));

        match sign_ver.as_str() {
            "1.1" => Auth11::new(path, &keypath, method, &userid, "1", Some(body.clone().into()))
                .build(request.headers_mut())?,
            _ => Auth13::new(path, &keypath, method, &userid, "1", Some(body.clone().into()))
                .build(request.headers_mut())?,
        };

        let mut core = Core::new()?;
        let client = HyperClient::configure()
            .connector(HttpsConnector::new(4, &core.handle())?)
            .build(&core.handle());


        let json = APPLICATION_JSON;
        request.headers_mut().set(Accept(vec![qitem(json.clone())]));
        request.headers_mut().set(ContentType::json());
        request.headers_mut().set(ContentLength(body.clone().len() as u64));
        request.headers_mut().set(OpsApiInfo(1));
        request.headers_mut().set(OpsApiVersion(1));
        request
            .headers_mut()
            .set(ChefVersion(String::from("13.3.34")));

        request.set_body(body);

        let resp = client.request(request).and_then(|res| {
            res.body().concat2().and_then(move |body| {
                serde_json::from_slice(&body)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e).into())
            })
        });

        core.run(resp).map_err(|e| e.into())
    }
}
