use rocket::request::FromRequest;
use rocket::Outcome;
use rocket::Request;
use syntect::parsing::{SyntaxSet, SyntaxReference};

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
        F: Fn(&&SyntaxReference) -> bool
{
    ss
        .syntaxes()
        .iter()
        .find(predicate)
}
