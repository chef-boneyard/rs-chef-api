import!();
requests!(DataQuery, data);

impl<'c> DataQuery<'c> {
    path!(data_bag);
    path!(item);
    acls!();
}
