import!();
requests!(RolesQuery, roles);

impl<'c> RolesQuery<'c> {
    path!(role);
    path!(-> environments);
    path!(environment);

    acls!();
}
