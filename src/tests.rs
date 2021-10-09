use once_cell::sync::Lazy;
use rocket::futures;

use super::rocket;
use rocket::http::Header;
use rocket::http::Status;
use rocket::local::asynchronous::Client;

/// Global mutable singleton of the LocalClient otherwise multiple tests fail
static CLIENT: Lazy<Client> = Lazy::new(|| futures::executor::block_on(async {
    Client::tracked(rocket())
        .await
        .expect("Valid rocket instance")
}));

/// This is just for convenience
static HOST_HEADER: Lazy<Header> = Lazy::new(|| Header::new("Host", "localhost:8000"));

#[rocket::async_test]
async fn test_index() {
    let client: &Client = &*CLIENT;

    let mut req = client.get("/");
    req.add_header(HOST_HEADER.clone());
    let res = req.dispatch().await;
    assert_eq!(res.status(), Status::Ok);
    assert!(res
        .into_string()
        .await
        .expect("Content in body")
        .contains("The pastebin that hopefully doesn't suck"));
}

#[rocket::async_test]
async fn test_gui() {
    let client: &Client = &*CLIENT;

    let mut req = client.get("/gui");
    req.add_header(HOST_HEADER.clone());
    let res = req.dispatch().await;
    assert_eq!(res.status(), Status::Ok);
    assert!(res
        .into_string()
        .await
        .expect("Content in body")
        .contains("<h1><a href=\"/\">pastor</a> - gui</h1>"));
}

#[rocket::async_test]
async fn test_create() {
    let client: &Client = &*CLIENT;

    // 1. Upload
    let mut req_upload = client.post("/");
    req_upload.add_header(HOST_HEADER.clone());
    req_upload.add_header(Header::new(
        "Content-Type",
        "multipart/form-data; boundary=---------------------------40655189221862374922500070259",
    ));
    req_upload.set_body("-----------------------------40655189221862374922500070259\r\nContent-Disposition: form-data; name=\"paste-content\"\r\n\r\nthis is a test\r\n-----------------------------40655189221862374922500070259--\r\n");
    let res_upload = req_upload.dispatch().await;
    assert_eq!(res_upload.status(), Status::Ok);
    let res_body = res_upload.into_string().await.expect("Content in body");
    // Notice that localhost also starts with https
    assert!(res_body.starts_with(&format!("https://{}", HOST_HEADER.value())));

    let paste_id = res_body.split(" ").collect::<Vec<&str>>()[0]
        .split("/")
        .collect::<Vec<&str>>()[3]
        .split(".")
        .collect::<Vec<&str>>()[0];

    // 2. Download
    let req_download = client.get(format!("/{}", paste_id));
    let res_download = req_download.dispatch().await;
    assert_eq!(res_download.status(), Status::Ok);
    let res_download_body = res_download.into_string().await.expect("Content in body");
    assert_eq!(res_download_body, "this is a test");
}
