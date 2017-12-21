import!();
requests!(UsersQuery, users);

impl<'c> UsersQuery<'c> {
    path!(user);
    path!(-> keys);
    path!(key);
}
