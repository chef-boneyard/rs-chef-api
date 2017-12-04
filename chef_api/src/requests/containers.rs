import!();
requests!(ContainersQuery, containers);

impl<'c> ContainersQuery<'c> {
    path!(container);
    acls!();
}
