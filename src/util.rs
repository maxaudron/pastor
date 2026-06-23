use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::Request;
use syntect::parsing::{SyntaxReference, SyntaxSet};

use phf::phf_map;

use async_trait::async_trait;

pub static MIME_EXT: phf::Map<&'static str, &'static str> = phf_map! {
    "text/plain" => "txt", // This one might be unnecessary
    "image/png" => "png",
    "image/jpeg" => "jpg",
    "application/x-shellscript" => "sh",
};

// pub struct HostHeader<'a>(pub &'a str);
//
// #[async_trait]
// impl<'r> FromRequest<'r> for HostHeader<'r> {
//     type Error = ();
//
//     async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
//         match request.headers().get_one("Host") {
//             Some(h) => Outcome::Success(HostHeader(h)),
//             None => Outcome::Forward(()),
//         }
//     }
// }

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
    match MIME_EXT.get(mime) {
        Some(ext) => Some(ext.to_string()),
        None => {
            // 2. Check if mime_guess returns exactly one result
            match mime_guess::get_mime_extensions_str(mime) {
                Some(guesses) if guesses.len() == 1 => Some(guesses[0].to_string()),
                _ => {
                    // 3. If all fails, use no extension at all
                    None
                }
            }
        }
    }
}
