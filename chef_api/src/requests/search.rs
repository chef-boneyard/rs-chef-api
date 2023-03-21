import!();
requests!(SearchQuery, search);

impl<'c> SearchQuery<'c> {
    path!(search_index);

    pub fn q(&mut self, query: &str) -> &mut Self {
        self.q = Some(query.to_owned());
        self
    }
}
