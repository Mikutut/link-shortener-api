use rocket::{catch};
use crate::responses;
use chrono::Utc;
use rocket::{State, Request, outcome::Outcome::*, http::{Header, Status}, Responder, serde::json::{Json, Value}};
use crate::fairings::rate_limit::RateLimitState;
use crate::config;
use std::sync::Mutex;

#[derive(Responder)]
pub struct RateLimitedWrappingResponder<'h, R> {
  inner: R,
  retry_after: Header<'h>
}

#[catch(429)]
pub async fn rate_limited<'a>(req: &Request<'_>) -> Result<RateLimitedWrappingResponder<'a, responses::new::ResponseResult<Json<responses::new::JsonErrorResponse<()>>>>, responses::new::ResponseResult<Json<responses::new::JsonErrorResponse<()>>>> {
  use responses::new::*;
  let mut response_builder: ResponseBuilder<(), Value> = ResponseBuilder::new();

  response_builder.error(
    Status::TooManyRequests,
    ResponseErrorType::RateLimitedError,
    String::from("You have been rate limited! Refer to 'Retry-After' header for cooldown duration.")
  );

  let inner = response_builder.build().json_respond();

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

#[catch(422)]
pub fn invalid_request_data() -> responses::new::ResponseResult<Json<responses::new::JsonErrorResponse<()>>> {
  use responses::new::*;

  let mut response_builder = ResponseBuilder::new();

  response_builder.error(
    Status::UnprocessableEntity,
    ResponseErrorType::ValidationError,
    String::from("Could not process request. Make sure your request body is of correct format.")
  );

  response_builder.build().json_respond()
}

#[catch(default)]
pub fn default_catcher(status: Status, _req: &Request) -> responses::new::ResponseResult<Json<responses::new::JsonErrorResponse<()>>> {
  use responses::new::*;

  let mut response_builder = ResponseBuilder::new();

  response_builder.error(
    status,
    ResponseErrorType::UndefinedError,
    String::from("Could not process request. Contact the administrator.")
  );
  
  response_builder.build().json_respond()
} 