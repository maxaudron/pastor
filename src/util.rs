use std::io::Cursor;

use rocket::{http::{ContentType, Status}, request::FromRequest, response::Body};
use rocket::Outcome;
use rocket::Request;
use rocket::Response;
use syntect::parsing::{SyntaxReference, SyntaxSet};

use crate::dict::DICT_MIME_EXT;

pub struct HostHeader<'a>(pub &'a str);
impl<'a, 'r> FromRequest<'a, 'r> for HostHeader<'a> {
    type Error = ();

    fn from_request(request: &'a Request) -> rocket::request::Outcome<Self, Self::Error> {
        match request.headers().get_one("Host") {
            Some(h) => Outcome::Success(HostHeader(h)),
            None => Outcome::Forward(()),
        }
    }
}

pub fn expires(size: u64) -> i64 {
    let min_age = 5.0;
    let max_age = 365.0;
    let max_size = 512.0;

    let size: f64 = ((size / 1024) / 1024) as f64;

    let mut expiry = min_age + (-max_age + min_age) * (size / max_size - 1.0).powf(3.0);

    if expiry < 5.0 {
        expiry = 5.0
    };

    (expiry * 86400.0) as i64
}

pub fn find_syntax_by_name<F>(ss: &SyntaxSet, predicate: F) -> Option<&SyntaxReference>
where
    F: Fn(&&SyntaxReference) -> bool,
{
    ss.syntaxes().iter().find(predicate)
}

pub fn ext_from_mime(mime: &str) -> Option<String> {
    // 1. Check if our well-known mime types have an entry
    match DICT_MIME_EXT.get(mime) {
        Some(ext) => Some(String::from(".") + ext),
        None => {
            // 2. Check if mime_guess returns exactly one result
            match mime_guess::get_mime_extensions_str(mime) {
                Some(guesses) if guesses.len() == 1 => {
                    Some(String::from(".") + guesses[0])
                },
                _ => {
                    // 3. If all fails, use no extension at all
                    None
                }
            }
        },
    }
}

pub fn create_response_from_string(content: String, content_type: Option<ContentType>) -> Response<'static> {
    let mut res = Response::new();
    res.set_status(Status::Ok);
    res.set_header(content_type.unwrap_or(ContentType::HTML));
    let size = content.len() as u64;
    let body = Body::Sized(Cursor::new(content), size);
    res.set_raw_body(body);
    res
}
