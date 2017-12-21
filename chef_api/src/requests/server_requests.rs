import!();
requests!(root ServerRequestsQuery);

impl<'c> ServerRequestsQuery<'c> {
    path!(-> status = "_status");
    path!(-> authenticate_user);
    path!(-> system_recovery);
    path!(-> license);
    path!(-> server_api_version);
}
