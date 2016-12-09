use authentication::Authenticator;
use authentication::auth11::Auth11;
use authentication::auth13::Auth13;
use config::Config;
use http_headers::*;
use hyper::client::Response as HyperResponse;
use hyper::Client as HyperClient;
use hyper::header::{Accept, ContentType, qitem};
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::client::IntoUrl;
use hyper::header::Headers;
use hyper::net::HttpsConnector;
use hyper_openssl::OpensslClient;
use std::io::Read;
use serde_json;
use serde::ser::Serialize;
use serde::de::Deserialize;
use errors::*;

#[derive(Debug,Clone)]
pub struct ApiClient {
    pub config: Config,
}

#[derive(Debug)]
pub struct Response {
    pub hyper_response: HyperResponse,
    pub body: String,
}

impl Response {
    fn from_hyper_response(mut hyper_response: HyperResponse) -> Result<Response> {
        let mut body = String::new();
        hyper_response.read_to_string(&mut body).map_err(|e| e.into()).map(|_| {
            Response {
                hyper_response: hyper_response,
                body: body,
            }
        })
    }

    pub fn from_json<T: Deserialize>(&self) -> Result<T> {
        serde_json::from_str(&*self.body).chain_err(|| "Failed to decode json")
    }
}

impl ApiClient {
    pub fn new(config: Config) -> ApiClient {
        ApiClient { config: config }
    }

    pub fn from_json_config(pth: &str) -> ApiClient {
        Config::from_json(pth)
            .map(|cfg| ApiClient::new(cfg))
            .unwrap()
    }

    pub fn config(mut self, config: Config) -> ApiClient {
        self.config = config;
        self
    }

    pub fn get(&self, path: &str) -> Result<Response> {
        self.send_with_body(path, "", "get")
    }

    pub fn delete(&self, path: &str) -> Result<Response> {
        self.send_with_body(path, "", "delete")
    }

    pub fn post<B>(&self, path: &str, body: B) -> Result<Response>
        where B: Serialize
    {
        let body = try!(serde_json::to_string(&body));
        self.send_with_body(path, body.as_ref(), "post")
    }

    pub fn put<B>(&self, path: &str, body: B) -> Result<Response>
        where B: Serialize
    {
        let body = try!(serde_json::to_string(&body));
        self.send_with_body(path, body.as_ref(), "put")
    }

    fn send_with_body(&self, path: &str, body: &str, method: &str) -> Result<Response> {
        let userid = self.config.user.clone().unwrap();
        let keypath = self.config.keypath.clone().unwrap();
        let sign_ver = self.config.sign_ver.clone();

        let headers: Headers = match sign_ver.as_str() {
            "1.1" => {
                try!(Auth11::new(path, &keypath, method, &userid, "1", Some(body.into())).headers())
            }
            _ => {
                try!(Auth13::new(path, &keypath, method, &userid, "1", Some(body.into())).headers())
            }
        };

        let url = try!(format!("{}{}", &self.config.url_base(), path).into_url());

        let mth = match method {
            "put" => Method::Put,
            "post" => Method::Post,
            "delete" => Method::Delete,
            "head" => Method::Head,
            _ => Method::Get,
        };

        let ssl = OpensslClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        let client = HyperClient::with_connector(connector);
        let client = client.request(mth, url);
        let client = client.body(body.as_bytes());

        let mut headers = headers.clone();

        let json = Mime(TopLevel::Application, SubLevel::Json, vec![]);
        headers.set(Accept(vec![qitem(json.clone())]));
        headers.set(ContentType(json));
        headers.set(OpsApiInfo(1));
        headers.set(OpsApiVersion(1));
        headers.set(ChefVersion(String::from("12.5.1")));

        let client = client.headers(headers);

        let resp = try!(client.send());
        let resp = try!(Response::from_hyper_response(resp));

        if resp.hyper_response.status.is_success() {
            Ok(resp)
        } else {
            Err(ErrorKind::UnsuccessfulResponse(resp).into())
        }
    }
}
