import!();
requests!(SearchQuery, search);

impl<'c> SearchQuery<'c> {
    path!(search_index);
}
