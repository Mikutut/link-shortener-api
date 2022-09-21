//TODO: Replace all occurrences of old response system
use rocket::{catch};
use crate::responses::{ResponseError, ResponseErrorType, Error};
use chrono::Utc;
use rocket::{State, Request, outcome::Outcome::*, http::{Header, Status}, Responder, serde::json::Json};
use crate::fairings::rate_limit::RateLimitState;
use crate::config;
use std::sync::Mutex;

#[derive(Responder)]
pub struct RateLimitedWrappingResponder<'h, R> {
  inner: R,
  retry_after: Header<'h>
}

//TODO: Add new responses
#[catch(429)]
pub async fn rate_limited<'a>(req: &Request<'_>) -> Result<RateLimitedWrappingResponder<'a, Error>, Error> {
  let inner = (
    Status::TooManyRequests,
    Json(ResponseError::new(
      ResponseErrorType::RateLimitedError,
      "You have been rate limited!".into()
    ))
  );

  match req.client_ip() {
    Some(ip) => {
      match req.guard::<&State<config::Config>>().await {
        Success(config) => {
          match req.guard::<&State<Mutex<RateLimitState>>>().await {
            Success(state) => {
              match state.lock() {
                Ok(lock) => {
                  match lock.get(&ip) {
                    Some(entry) => {
                      let time_window = config.max_requests_time_window.clone();
                      let date = entry.1.clone();

                      let response = RateLimitedWrappingResponder {
                        inner: inner,
                        retry_after: Header::new("Retry-After", format!("{}", 
                          (time_window - Utc::now().naive_utc().signed_duration_since(date).num_seconds())
                        ))
                      };

                      Ok(response)
                    },
                    None => {
                      Err(inner)
                    }
                  }
                },
                Err(_) => {
                  Err(inner)
                }
              }
            },
            _ => {
              Err(inner)
            }
          }
        },
        _ => {
          Err(inner)
        }
      }
    }
    None => {
      Err(inner)
    }
  }
  // let time_window = config.max_requests_time_window.clone();
  // let retry_after = Header::new("Retry-After", format!("{}", 

  // ));
}

//TODO: Add new responses
#[catch(422)]
pub fn invalid_request_data() -> Error {
  (
    Status::UnprocessableEntity,
    Json(ResponseError::new(
      ResponseErrorType::ValidationError,
      "Could not process request. Make sure your request body is of correct format.".into()
    ))
  )
}

//TODO: Add new responses
#[catch(default)]
pub fn default_catcher(status: Status, _req: &Request) -> Error {
  (
    status,
    Json(ResponseError::new(
      ResponseErrorType::UndefinedError,
      "Could not process request. Contact the administrator.".into()
    ))
  )
} 