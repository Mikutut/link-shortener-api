pub mod rate_limit {
  use chrono::Utc;
  use rocket::{request::{FromRequest, Outcome, Request}, http::Status, State};
  use crate::{fairings::rate_limit, config};

  #[derive(Debug)]
  pub enum RateLimit {
    Allowed,
    Rejected(i64),
    Error
  }

  #[rocket::async_trait]
  impl<'r> FromRequest<'r> for RateLimit {
    type Error = RateLimit;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, <crate::guards::rate_limit::RateLimit as FromRequest<'r>>::Error> {
      match req.guard::<&State<config::Config>>().await {
        Outcome::Success(config) => match req.guard::<&State<rate_limit::RateLimitState>>().await {
          Outcome::Success(state) => {
            let max_requests = config.max_requests;
            let time_window = config.max_requests_time_window;
            let current_time = Utc::now().naive_utc();

            match req.client_ip() {
              Some(ip) => match state.lock() {
                Ok(mut lock) => match lock.get_mut(&ip) {
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
                    println!("Could not find client IP's entry in rate limit state!");
                    Outcome::Failure((Status::InternalServerError, RateLimit::Error))
                  }
                },
                Err(_) => {
                  println!("Could not acquire lock on rate limit state!");
                  Outcome::Failure((Status::InternalServerError, RateLimit::Error))
                }
              },
              None => {
                println!("Could not get client's IP!");
                Outcome::Failure((Status::InternalServerError, RateLimit::Error))
              }
            }
          },
          _ => {
            println!("Could not acquire rate limit state!");
            Outcome::Failure((Status::InternalServerError, RateLimit::Error))
          }
        },
        _ => {
          println!("Could not acquire config!");
          Outcome::Failure((Status::InternalServerError, RateLimit::Error))
        }
      }
      
    }
  }
}