use rocket::request::FromRequest;
use rocket::Outcome;
use rocket::Request;

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
