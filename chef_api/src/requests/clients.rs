import!();
requests!(ClientsQuery, clients);

impl<'c> ClientsQuery<'c> {
    path!(client);
    path!(-> keys);
    path!(key);

    acls!();
}
