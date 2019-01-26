use crate::credentials::Config;

use hyper::client::HttpConnector;
use hyper::Client as HyperClient;
use hyper_openssl::HttpsConnector;

use tokio_core::reactor::Core;

use failure::Error;
use serde::ser::*;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

use crate::requests::*;

/// Struct used to start requests to the Chef Server API.
#[derive(Debug, Clone)]
pub struct ApiClient {
    /// Configuration for the client, containing credentials
    pub config: Config,
    /// The Hyper HTTP Client.
    pub client: Rc<HyperClient<HttpsConnector<HttpConnector>>>,
    /// The async core
    pub core: Rc<RefCell<Core>>,
}

impl ApiClient {
    /// Create a new ApiClient struct. It takes a `Config` type. Typically one would use
    /// `from_credentials` rather than calling this directly.
    pub fn new(config: Config) -> Result<Self, Error> {
        let core = Core::new()?;
        let handle = core.handle();

        let client = HyperClient::configure()
            .connector(HttpsConnector::new(4, &handle)?)
            .build(&handle);

        Ok(Self {
            config,
            core: Rc::new(RefCell::new(core)),
            client: Rc::new(client),
        })
    }

    /// Create a new `ApiClient` struct using a set of credentials as defined in Chef RFC 99.
    ///
    /// Accepts an `Option<String>` containing the name of a credentials profile.
    pub fn from_credentials(profile: Option<&str>) -> Result<Self, Error> {
        Config::from_credentials(profile).map(ApiClient::new)?
    }

    build!(clients, ClientsQuery);
    build!(containers, ContainersQuery);
    build!(controls, ControlsQuery);
    build!(cookbook_artifacts, CookbookArtifactsQuery);
    build!(cookbooks, CookbooksQuery);
    build!(data, DataQuery);
    build!(environments, EnvironmentsQuery);
    build!(groups, GroupsQuery);
    build!(nodes, NodesQuery);
    build!(policies, PoliciesQuery);
    build!(policy_groups, PolicyGroupsQuery);
    build!(principals, PrincipalsQuery);
    build!(roles, RolesQuery);
    build!(sandboxes, SandboxesQuery);
    build!(search, SearchQuery);
    build!(universe, UniverseQuery);
    build!(users, UsersQuery);

    build!(server_organizations, ServerOrganizationsQuery);
    build!(server_users, ServerUsersQuery);
    build!(server_requests, ServerRequestsQuery);
}

/// The Execute trait finalises and executes a request, by making the desired HTTP call.
pub trait Execute {
    /// Make an HTTP `get` request.
    fn get(&self) -> Result<Value, Error> {
        self.execute(None::<String>, "get")
    }

    /// Make an HTTP `delete` request.
    fn delete(&self) -> Result<Value, Error> {
        self.execute(None::<String>, "delete")
    }

    /// Make an HTTP `head` request.
    fn head(&self) -> Result<Value, Error> {
        self.execute(None::<String>, "head")
    }

    /// Make an HTTP `post` request, accepting a `T` which can be converted into JSON.
    fn post<B>(&self, body: &B) -> Result<Value, Error>
    where
        B: Serialize,
    {
        self.execute(Some(body), "post")
    }

    /// Make an HTTP `put` request, accepting a `T` which can be converted into JSON.
    fn put<B>(&self, body: &B) -> Result<Value, Error>
    where
        B: Serialize,
    {
        self.execute(Some(body), "put")
    }

    /// Set the Chef API Version.
    fn api_version(&mut self, api_version: &str) -> &mut Self;

    #[doc(hidden)]
    fn execute<B>(&self, body: Option<B>, method: &str) -> Result<Value, Error>
    where
        B: Serialize;
}
