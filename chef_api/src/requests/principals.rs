import!();
requests!(PrincipalsQuery, principals);

impl<'c> PrincipalsQuery<'c> {
    path!(principal);
}
