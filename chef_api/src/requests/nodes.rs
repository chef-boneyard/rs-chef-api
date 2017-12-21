import!();
requests!(NodesQuery, nodes);

impl<'c> NodesQuery<'c> {
    path!(node);
    acls!();
}
