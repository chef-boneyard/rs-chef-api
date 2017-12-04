import!();
requests!(GroupsQuery, groups);

impl<'c> GroupsQuery<'c> {
    path!(group);
    acls!();
}
