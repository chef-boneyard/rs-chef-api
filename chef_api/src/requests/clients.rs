import!();
requests!(ClientsQuery, clients);

impl<'c> ClientsQuery<'c> {
    path!(
        /// Operate on an individual Chef client.
        ///
        /// - GET: Retrieve a client
        /// - PUT: Update an existing client
        /// - DELETE: Delete an existing client
        ///
        /// ```rust,no_run
        /// # use chef_api::api_client::*;
        /// # let api = ApiClient::from_credentials(None).unwrap();
        /// let client = api.clients().client("my_client").get();
        /// ```
        client
    );

    path!(
        /// Get all of a specified client's keys.
        ///
        /// Can only be called once a client has been specified with `client()`.
        ///
        /// - GET: Retrieve a list of keys
        /// - POST: Add a new key
        ///
        /// ```rust,no_run
        /// # use chef_api::api_client::*;
        /// # let api = ApiClient::from_credentials(None).unwrap();
        /// let keys = api.clients().client("my_client").keys().get();
        /// ```
        -> keys);

    path!(
        /// Get a specified key.
        ///
        /// Can only be called once a client has been specified with `client()`.
        ///
        /// - GET: Retrieve a key
        /// - PUT: Update a key
        /// - DELETE: Delete a key
        ///
        /// ```rust,no_run
        /// # use chef_api::api_client::*;
        /// # let api = ApiClient::from_credentials(None).unwrap();
        /// let key = api.clients().client("my_client").keys().key("default").get();
        /// ```
        key
    );

    acls!();
}
