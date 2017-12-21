import!();
requests!(CookbookArtifactsQuery, cookbook_artifacts);

impl<'c> CookbookArtifactsQuery<'c> {
    path!(cookbook);
    path!(identifier);
}
