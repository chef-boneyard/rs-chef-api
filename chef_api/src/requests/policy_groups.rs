import!();
requests!(PolicyGroupsQuery, policy_groups);

impl<'c> PolicyGroupsQuery<'c> {
    path!(policy_group);

    pub fn policy(&mut self, policy: &str) -> &mut Self {
        self.path = add_path_element(self.path.clone(), &format!("policies/{}", policy));
        self
    }
}
