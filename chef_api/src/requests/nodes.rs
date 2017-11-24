import!();
requests!(NodesQuery, nodes);

impl<'c> NodesQuery<'c> {
    path!(name);
    acls!();
}
