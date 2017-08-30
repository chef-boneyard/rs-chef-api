use openssl;
use std;
use serde_json;
use url;
use hyper;

error_chain! {
    foreign_links {
        OpenSSL(openssl::error::ErrorStack);
        IOError(std::io::Error);
        JsonError(serde_json::Error);
        UrlParseError(url::ParseError);
        UriError(hyper::error::UriError);
        HTTPError(hyper::error::Error);
    }

    errors {
        PrivateKeyError(path: String) {
            description("Failed to read private key")
                display("Failed to read private key at {}", path)
        }
        ListError {
            description("Failed to interpret a list of items")
        }
        KeyMissingError(field: String) {
            description("Failed to fetch field from JSON")
                display("Failed to fetch {} from JSON", field)
        }
        UnparseableConfigError(path: String) {
            description("Can't read config file")
                display("Can't read config file at {}", path)
        }
    }
}
