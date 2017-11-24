macro_rules! build {
    ($name:ident, $type:ident) => {

        pub fn $name(&self) -> $type {
            self.into()
        }
    }
}

macro_rules! import {
    () => {
        use api_client::*;
        use authentication::auth11::Auth11;
        use authentication::auth13::Auth13;
        use config::Config;
        use failure::Error;
        use http_headers::*;
        use utils::add_path_element;

        use serde_json;
        use serde::de::DeserializeOwned;
        use serde::Serialize;

        use std::io;
        use std::rc::Rc;
        use std::cell::RefCell;

        use tokio_core::reactor::Core;
        use futures::{Future, Stream};

        use hyper::Client as HyperClient;
        use hyper::{Method, Request};
        use hyper::client::HttpConnector;
        use hyper::header::{qitem, Accept, ContentLength, ContentType};
        use hyper_openssl::HttpsConnector;
        use hyper::mime::APPLICATION_JSON;
    }
}

macro_rules! path {
    ($n:ident) => {

        pub fn $n(&mut self, add: &str) -> &mut Self {
            self.path = add_path_element(self.path.clone(), add);
            self
        }

    }
}


macro_rules! acls {
    () => {
        pub fn acl(&mut self, name: &str) -> &mut Self {
            self.path = add_path_element(self.path.clone(), name);
            self
        }

        pub fn permission(&mut self, p: &str) -> &mut Self {
            self.path = add_path_element(self.path.clone(), p);
            self
        }
    }
}

macro_rules! requests {
    ($n:ident, $p:tt) => {

        pub struct $n<'c> {
            pub(crate) client: &'c Rc<HyperClient<HttpsConnector<HttpConnector>>>,
            pub(crate) core: &'c Rc<RefCell<Core>>,
            pub(crate) config: &'c Config,
            pub(crate) path: String,
        }

        impl<'c> From<&'c ApiClient> for $n<'c> {
            fn from(api: &'c ApiClient) -> Self {
                let path = add_path_element(api.config.organization_path(), stringify!($p));
                Self {
                    config: &api.config,
                    client: &api.client,
                    core: &api.core,
                    path: path,
                }
            }
        }

        impl<'e> Execute for $n<'e> {
            fn execute<B, T>(&self, body: Option<B>, method: &str) -> Result<T, Error>
                where
                    B: Serialize,
                    T: DeserializeOwned
                    {
                        let userid = self.config.user.clone().unwrap();
                        let keypath = self.config.keypath.clone().unwrap();
                        let sign_ver = self.config.sign_ver.clone();
                        let path = self.path.clone();

                        let url = try!(format!("{}{}", &self.config.url_base(), path).parse());

                        let mth = match method {
                            "put" => Method::Put,
                            "post" => Method::Post,
                            "delete" => Method::Delete,
                            "head" => Method::Head,
                            _ => Method::Get,
                        };

                        let mut request = Request::new(mth, url);

                        let body = match body {
                            Some(b) => serde_json::to_string(&b)?,
                            None => String::from("")
                        };

                        match sign_ver.as_str() {
                            "1.1" => Auth11::new(
                                &path,
                                &keypath,
                                method,
                                &userid,
                                "1",
                                Some(body.clone().into()),
                                ).build(request.headers_mut())?,
                            _ => Auth13::new(
                                &path,
                                &keypath,
                                method,
                                &userid,
                                "1",
                                Some(body.clone().into()),
                                ).build(request.headers_mut())?,
                        };

                        let json = APPLICATION_JSON;
                        request.headers_mut().set(Accept(vec![qitem(json.clone())]));
                        request.headers_mut().set(ContentType::json());
                        request
                            .headers_mut()
                            .set(ContentLength(body.clone().len() as u64));
                        request.headers_mut().set(OpsApiInfo(1));
                        request.headers_mut().set(OpsApiVersion(1));
                        request
                            .headers_mut()
                            .set(ChefVersion(String::from("13.3.34")));

                        request.set_body(body);

                        let client = self.client;
                        let resp = client.request(request).and_then(|res| {
                            res.body().concat2().and_then(move |body| {
                                serde_json::from_slice(&body)
                                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e).into())
                            })
                        });

                        let mut core = self.core.try_borrow_mut()?;
                        core.run(resp).map_err(|e| e.into())
                    }
        }
    }
}
