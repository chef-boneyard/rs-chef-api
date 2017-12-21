import!();
requests!(EnvironmentsQuery, environments);

impl<'c> EnvironmentsQuery<'c> {
    path!(environment);
    path!(-> cookbooks);
    path!(cookbook);
    path!(-> cookbook_versions);
    path!(-> nodes);
    path!(-> recipes);
    path!(-> roles);
    path!(role);

    acls!();
}
