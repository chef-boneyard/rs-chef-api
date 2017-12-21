import!();
requests!(PoliciesQuery, policies);

impl<'c> PoliciesQuery<'c> {
    path!(policy);
    path!(-> revisions);
    path!(revision);
    acls!();
}
