use rocket::{request::{self, FromRequest, Outcome, Request}, http::Status};
use crate::fairings;

#[derive(Debug)]
pub enum RateLimit {
  Allowed,
  Rejected(i64)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimit {
  type Error = RateLimit;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    Outcome::Failure((Status::ServiceUnavailable, RateLimit::Rejected(i64::MAX)))
  }
}
