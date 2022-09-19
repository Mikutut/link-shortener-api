use chrono::Utc;
use rocket::{request::{self, FromRequest, Outcome, Request}, http::Status, State};
use crate::{fairings::rate_limit, config};
use std::sync::Mutex;

#[derive(Debug)]
pub enum RateLimit {
  Allowed,
  Rejected(i64)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimit {
  type Error = RateLimit;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    //Outcome::Failure((Status::ServiceUnavailable, RateLimit::Rejected(i64::MAX)))
    match req.guard::<&State<config::Config>>().await {
      Outcome::Success(config) => {
        match req.guard::<&State<Mutex<rate_limit::RateLimitState>>>().await {
          Outcome::Success(state) => {
            let max_requests: i64 = config.max_requests.clone();
            let time_window: i64 = config.max_requests_time_window.clone();
            let current_time = Utc::now().naive_utc();

            match req.client_ip() {
              Some(ip) => {
                let mut lock = state.lock().expect("lock rate limit state in request");

                match lock.get_mut(&ip) {
                  Some(entry) => {
                    let diff = current_time.signed_duration_since(entry.1).num_seconds();

                    if entry.0 >= max_requests {
                      Outcome::Failure((Status::TooManyRequests, RateLimit::Rejected(time_window - diff)))
                    } else {
                      entry.0 = entry.0 + 1;
                      Outcome::Success(RateLimit::Allowed)
                    }
                  },
                  None => {
                    println!("Client IP entry in rate limit state not found!");
                    Outcome::Failure((Status::InternalServerError, RateLimit::Rejected(i64::MAX)))
                  }
                }
              },
              None => {
                println!("Client IP is unknown!");
                Outcome::Failure((Status::InternalServerError, RateLimit::Rejected(i64::MAX)))
              }
            }
          },
          _ => {
            println!("Could not acquire rate limit state!");
            Outcome::Failure((Status::InternalServerError, RateLimit::Rejected(i64::MAX)))
          }
        }
      },
      _ => {
        println!("Could not acquire config!");
        Outcome::Failure((Status::InternalServerError, RateLimit::Rejected(i64::MAX)))
      }
    }
  }
}