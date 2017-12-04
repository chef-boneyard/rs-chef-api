import!();
requests!(root ServerUsersQuery, users);

impl<'c> ServerUsersQuery<'c> {
    path!(user);
    path!(-> keys);
    path!(key);

    path!(-> association_requests);
    path!(-> count);
    path!(association_request);

    acls!();
}
