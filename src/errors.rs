use openssl;
use std::io;
use serde_json;
use url;
use hyper;
use api_client::Response;

error_chain! {
    foreign_links {
        openssl::error::ErrorStack, OpenSSL;
        io::Error, IOError;
        serde_json::Error, JsonError;
        url::ParseError, UrlParseError;
        hyper::error::Error, HTTPError;
    }

    errors {
        PrivateKeyError(path: String) {
            description("Failed to read private key")
                display("Failed to read private key at {}", path)
        }
        ListError {
            description("Failed to interpret a list of items")
        }
        UnsuccessfulResponse(resp: Response) {
            description("Got a bad response from the Chef Server")
        }
    }
}
