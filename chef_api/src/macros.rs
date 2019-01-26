macro_rules! build {
    ($name:ident, $type:ident) => {
        #[doc="Generate a new $type request."]
        pub fn $name(&self) -> $type {
            self.into()
        }
    }
}

macro_rules! import {
    () => {
        use failure::Error;
        use $crate::api_client::*;
        use $crate::authentication::auth11::Auth11;
        use $crate::authentication::auth13::Auth13;
        use $crate::credentials::Config;
        use $crate::http_headers::*;
        use $crate::utils::add_path_element;

        use serde::Serialize;
        use serde_json;

        use std::cell::RefCell;
        use std::rc::Rc;

        use futures::{Future, Stream};
        use tokio_core::reactor::Core;

        use hyper::client::HttpConnector;
        use hyper::header::{qitem, Accept, ContentLength, ContentType};
        use hyper::mime::APPLICATION_JSON;
        use hyper::Client as HyperClient;
        use hyper::{Method, Request};
        use hyper_openssl::HttpsConnector;
    };
}

macro_rules! path {
    (
        $(#[$outer:meta])*
        -> $n:ident = $txt:tt
    ) => {
        $(#[$outer])*
        pub fn $n(&mut self) -> &mut Self {
            self.path = add_path_element(self.path.clone(), $txt);
            self
        }
    };
    (
        $(#[$outer:meta])*
        -> $n:ident
    ) => {
        $(#[$outer])*
        pub fn $n(&mut self) -> &mut Self {
            self.path = add_path_element(self.path.clone(), stringify!($n));
            self
        }
    };
    (
        $(#[$outer:meta])*
        $n:ident
    ) => {
        $(#[$outer])*
        pub fn $n(&mut self, value: &str) -> &mut Self {
            self.path = add_path_element(self.path.clone(), value);
            self
        }

    };
}

macro_rules! acls {
    () => {
        /// Get the list of ACLs on this object
        pub fn acl(&mut self) -> &mut Self {
            self.path = add_path_element(self.path.clone(), "_acl");
            self
        }

        /// Modify the given permission on the object.
        pub fn permission(&mut self, permission: &str) -> &mut Self {
            self.path = add_path_element(self.path.clone(), permission);
            self
        }
    }
}

macro_rules! request_type {
    ($n:ident) => {
        #[derive(Debug, Clone)]
        pub struct $n<'c> {
            pub(crate) client: &'c Rc<HyperClient<HttpsConnector<HttpConnector>>>,
            pub(crate) core: &'c Rc<RefCell<Core>>,
            pub(crate) config: &'c Config,
            pub(crate) path: String,
            pub(crate) api_version: String,
        }
    };
}

macro_rules! requests {
    (root $n:ident) => {
        request_type!($n);

        impl<'c> From<&'c ApiClient> for $n<'c> {
            fn from(api: &'c ApiClient) -> Self {
                let path = String::from("/");
                Self {
                    config: &api.config,
                    client: &api.client,
                    core: &api.core,
                    path,
                    api_version: String::from("1"),
                }
            }
        }

        execute!($n);
    };
    (root $n:ident, $p:tt) => {
        request_type!($n);

        impl<'c> From<&'c ApiClient> for $n<'c> {
            fn from(api: &'c ApiClient) -> Self {
                let path = add_path_element(String::from("/"), stringify!($p));
                Self {
                    config: &api.config,
                    client: &api.client,
                    core: &api.core,
                    path,
                    api_version: String::from("1"),
                }
            }
        }

        execute!($n);
    };
    ($n:ident, $p:tt) => {
        request_type!($n);

        impl<'c> From<&'c ApiClient> for $n<'c> {
            fn from(api: &'c ApiClient) -> Self {
                let path =
                    add_path_element(api.config.organization_path().unwrap(), stringify!($p));
                Self {
                    config: &api.config,
                    client: &api.client,
                    core: &api.core,
                    path,
                    api_version: String::from("1"),
                }
            }
        }

        execute!($n);
    };
}

macro_rules! execute {
    ($n:ident) => {
        use serde_json::Value;
        use $crate::errors::ChefError;

        impl<'e> Execute for $n<'e> {
            fn api_version(&mut self, api_version: &str) -> &mut Self {
                self.api_version = api_version.into();
                self
            }

            #[doc(hidden)]
            fn execute<B>(&self, body: Option<B>, method: &str) -> Result<Value, Error>
            where
                B: Serialize,
            {
                let userid = self.config.client_name()?;
                let key = self.config.key()?;
                let sign_ver = self.config.sign_ver.clone();
                let path = self.path.clone();
                let api_version = self.api_version.clone();

                let url = try!(format!("{}{}", &self.config.url_base()?, path).parse());

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
                    None => serde_json::to_string("")?,
                };

                match sign_ver.as_str() {
                    "1.1" => Auth11::new(
                        &path,
                        &key,
                        method,
                        &userid,
                        &api_version,
                        Some(body.clone().into()),
                    )
                    .build(request.headers_mut())?,
                    _ => Auth13::new(
                        &path,
                        &key,
                        method,
                        &userid,
                        &api_version,
                        Some(body.clone().into()),
                    )
                    .build(request.headers_mut())?,
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
                let resp = client
                    .request(request)
                    .map_err(ChefError::HTTPError)
                    .and_then(|res| {
                        debug!("Status is {:?}", res.status());

                        let status = res.status();
                        res.body()
                            .concat2()
                            .map_err(ChefError::HTTPError)
                            .and_then(move |body| {
                                let body: Value =
                                    serde_json::from_slice(&body).map_err(ChefError::JsonError)?;

                                if status.is_success() {
                                    Ok(body)
                                } else {
                                    Err(ChefError::ChefServerResponseError(status.as_u16()))
                                }
                            })
                    });

                let mut core = self.core.try_borrow_mut()?;
                core.run(resp).map_err(|e| e.into())
            }
        }
    };
}
