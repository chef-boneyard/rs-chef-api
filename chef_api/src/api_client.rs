use credentials::Config;

use hyper::Client as HyperClient;
use hyper::client::HttpConnector;
use hyper_openssl::HttpsConnector;

use tokio_core::reactor::Core;

use std::rc::Rc;
use std::cell::RefCell;
use serde::ser::*;
use serde::de::DeserializeOwned;
use serde_json::Value;
use failure::Error;

use requests::*;

#[derive(Debug, Clone)]
pub struct ApiClient {
    pub config: Config,
    pub client: Rc<HyperClient<HttpsConnector<HttpConnector>>>,
    pub core: Rc<RefCell<Core>>,
}

impl ApiClient {
    pub fn new(config: Config) -> Result<Self, Error> {
        let core = Core::new()?;
        let handle = core.handle();

        let client = HyperClient::configure()
            .connector(HttpsConnector::new(4, &handle)?)
            .build(&handle);

        Ok(Self {
            config: config,
            core: Rc::new(RefCell::new(core)),
            client: Rc::new(client),
        })
    }

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
    build!(users, UsersQuery);

    build!(server_organizations, ServerOrganizationsQuery);
    build!(server_users, ServerUsersQuery);
    build!(server_requests, ServerRequestsQuery);
}

pub trait Execute {
    fn get(&self) -> Result<Value, Error> {
        self.execute(None::<String>, "get")
    }

    fn delete(&self) -> Result<Value, Error> {
        self.execute(None::<String>, "delete")
    }

    fn head(&self) -> Result<Value, Error> {
        self.execute(None::<String>, "head")
    }

    fn post<B>(&self, body: &B) -> Result<Value, Error>
    where
        B: Serialize,
    {
        self.execute(Some(body), "post")
    }

    fn put<B>(&self, body: &B) -> Result<Value, Error>
    where
        B: Serialize,
    {
        self.execute(Some(body), "put")
    }

    fn api_version(&mut self, api_version: &str) -> &mut Self;

    fn execute<B, T>(&self, body: Option<B>, method: &str) -> Result<T, Error>
    where
        B: Serialize,
        T: DeserializeOwned;
}