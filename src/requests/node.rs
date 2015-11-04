use api_client::Request;
use hyper::method::Method;
use utils::squeeze_path;

#[derive(Debug)]
pub struct NodesRequest {
    path: String,
}

impl Request for NodesRequest {
    fn full_path(&self, base: &str) -> String {
        let pth = format!("{}/{}", base, self.path);
        squeeze_path(pth)
    }

    fn method(&self) -> Method {
        Method::Get
    }
}

impl NodesRequest {
    pub fn new() -> NodesRequest {
        NodesRequest { path: String::from("/nodes") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use api_client::*;
    use hyper::method::Method;

    #[test]
    fn test_nodes_method() {
        let nr = NodesRequest::new();
        assert_eq!(Method::Get, nr.method())
    }

    #[test]
    fn test_nodes_path() {
        let nr = NodesRequest::new();
        assert_eq!(nr.full_path("/organizations/clownco/"), "/organizations/clownco/nodes")
    }
}
