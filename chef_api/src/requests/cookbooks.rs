import!();
requests!(CookbooksQuery, cookbooks);

impl<'c> CookbooksQuery<'c> {
    path!(-> latest = "_latest");
    path!(-> recipes = "_recipes");
    path!(cookbook);
    path!(version);
    acls!();
}
