import!();
requests!(root ServerOrganizationsQuery, organizations);

impl<'c> ServerOrganizationsQuery<'c> {
    path!(organization);
    acls!();
}
