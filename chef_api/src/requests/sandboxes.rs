import!();
requests!(SandboxesQuery, sandboxes);

impl<'c> SandboxesQuery<'c> {
    path!(sandbox);
    acls!();
}
