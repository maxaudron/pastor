use once_cell::sync::Lazy;

use super::rocket;
use rocket::http::Header;
use rocket::local::Client;
use rocket::http::Status;

/// Global mutable singleton otherwise multiple tests fail
static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::new(rocket()).expect("Valid rocket instance")
});

static HOST_HEADER: Lazy<Header> = Lazy::new(|| {
    Header::new("Host", "localhost:8000")
});


#[test]
fn test_index() {
    let client: &Client = &*CLIENT;
    let mut req = client.get("/");
    req.add_header(HOST_HEADER.clone());
    let mut res = req.dispatch();
    assert_eq!(res.status(), Status::Ok);
    assert!(
        res
            .body_string()
            .expect("Content in body")
            .contains("The pastebin that hopefully doesn't suck")
    );
}

#[test]
fn test_gui() {
    let client: &Client = &*CLIENT;
    let mut req = client.get("/gui");
    req.add_header(HOST_HEADER.clone());
    let mut res = req.dispatch();
    assert_eq!(res.status(), Status::Ok);
    assert!(
        res
            .body_string()
            .expect("Content in body")
            .contains("<h1><a href=\"/\">pastor</a> - gui</h1>")
    );
}

