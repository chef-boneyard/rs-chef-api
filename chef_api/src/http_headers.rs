header! {
    (OpsSign, "X-Ops-Sign") => [String]
}
header! {
    (OpsUserId, "X-Ops-Userid") => [String]
}
header! {
    (OpsTimestamp, "X-Ops-Timestamp") => [String]
}
header! {
    (OpsContentHash, "X-Ops-Content-Hash") => [String]
}
header! {
    (OpsApiVersion, "X-Ops-Server-API-Version") => [u8]
}
header! {
    (OpsApiInfo, "X-Ops-Server-API-Info") => [u8]
}
header! {
    (ChefVersion, "X-Chef-Version") => [String]
}
