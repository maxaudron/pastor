use once_cell::sync::Lazy;

use super::rocket;
use rocket::http::Header;
use rocket::http::Status;
use rocket::local::Client;

/// Global mutable singleton of the LocalClient otherwise multiple tests fail
static CLIENT: Lazy<Client> = Lazy::new(|| Client::new(rocket()).expect("Valid rocket instance"));

/// This is just for convenience
static HOST_HEADER: Lazy<Header> = Lazy::new(|| Header::new("Host", "localhost:8000"));

#[test]
fn test_index() {
    let client: &Client = &*CLIENT;
    let mut req = client.get("/");
    req.add_header(HOST_HEADER.clone());
    let mut res = req.dispatch();
    assert_eq!(res.status(), Status::Ok);
    assert!(res
        .body_string()
        .expect("Content in body")
        .contains("The pastebin that hopefully doesn't suck"));
}

#[test]
fn test_gui() {
    let client: &Client = &*CLIENT;
    let mut req = client.get("/gui");
    req.add_header(HOST_HEADER.clone());
    let mut res = req.dispatch();
    assert_eq!(res.status(), Status::Ok);
    assert!(res
        .body_string()
        .expect("Content in body")
        .contains("<h1><a href=\"/\">pastor</a> - gui</h1>"));
}

#[test]
fn test_create() {
    let client: &Client = &*CLIENT;

    // 1. Upload
    let mut req_upload = client.post("/");
    req_upload.add_header(HOST_HEADER.clone());
    req_upload.add_header(Header::new(
        "Content-Type",
        "multipart/form-data; boundary=---------------------------40655189221862374922500070259",
    ));
    req_upload.set_body("-----------------------------40655189221862374922500070259\r\nContent-Disposition: form-data; name=\"paste-content\"\r\n\r\nthis is a test\r\n-----------------------------40655189221862374922500070259--\r\n");
    let mut res_upload = req_upload.dispatch();
    assert_eq!(res_upload.status(), Status::Ok);
    let res_body = res_upload.body_string().expect("Content in body");
    // Notice that localhost also starts with https
    assert!(res_body.starts_with(&format!("https://{}", HOST_HEADER.value())));

    let paste_id = res_body.split(" ").collect::<Vec<&str>>()[0]
        .split("/")
        .collect::<Vec<&str>>()[3]
        .split(".")
        .collect::<Vec<&str>>()[0];

    // 2. Download
    let req_download = client.get(format!("/{}", paste_id));
    let mut res_download = req_download.dispatch();
    assert_eq!(res_download.status(), Status::Ok);
    let res_download_body = res_download.body_string().expect("Content in body");
    assert_eq!(res_download_body, "this is a test");
}
