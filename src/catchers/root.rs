use chrono::Utc;
use rocket::{
  catch, 
  State, 
  Request, 
  http::{
    Header, 
    Status
  }, 
  Responder, 
  serde::json::{
    Json, 
    Value
  },
  outcome::Outcome::Success
};
use crate::responses::{ResponseErrorType, ResponseData, Response, errors};
use crate::fairings::rate_limit::RateLimitState;
use crate::config;

#[derive(Responder)]
pub struct RateLimitedWrappingResponder<'h, R> {
  inner: R,
  retry_after: Header<'h>
}

#[catch(429)]
pub async fn rate_limited<'a>(req: &Request<'_>) -> Result<RateLimitedWrappingResponder<'a, (Status, Json<Response<(), errors::RateLimitedError>>)>, (Status, Json<Response<(), Value>>)> {
  let mut response_data: ResponseData<(), Value> = ResponseData::new();

  response_data = response_data
    .set_status(Status::InternalServerError)
    .set_error_type(ResponseErrorType::UndefinedError)
    .set_error_message(String::from("You have been rate limited but server could not determine the length of your cooldown. Please wait for at least an hour and try again!"));

  match req.guard::<&State<config::Config>>().await {
    Success(config) => match req.client_ip() {
      Some(ip) => match req.guard::<&State<RateLimitState>>().await {
        Success(state) => match state.lock() {
            Ok(lock) => match lock.get(&ip) {
              Some(entry) => {
                let max_requests = config.max_requests.clone();
                let time_window = config.max_requests_time_window.clone();
                let date = entry.1.clone();
                let retry_after = time_window - Utc::now().naive_utc().signed_duration_since(date).num_seconds();

                let response_data = response_data
                  .set_status(Status::TooManyRequests)
                  .set_error_type(ResponseErrorType::RateLimitedError)
                  .set_error_message(String::from("You have been rate limited!"))
                  .transform_error::<errors::RateLimitedError>(errors::RateLimitedError {
                    max_requests: max_requests,
                    time_window: time_window,
                    cooldown: retry_after
                  });

                let response = RateLimitedWrappingResponder {
                  inner: response_data.to_response().json_respond(),
                  retry_after: Header::new("Retry-After", format!("{}", retry_after))
                };

                Ok(response)
              },
              None => Err(response_data.to_response().json_respond())
            },
            Err(_) => {
              Err(response_data.to_response().json_respond())
            } 
        },
        _ => {
          Err(response_data.to_response().json_respond())
        }
      },
      None => {
        Err(response_data.to_response().json_respond())
      }
    },
    _ => {
      Err(response_data.to_response().json_respond())
    }
  }
}

#[catch(422)]
pub fn invalid_request_data() -> (Status, Json<Response<(), ()>>) {
  ResponseData::<(), ()>::new()
    .set_status(Status::UnprocessableEntity)
    .set_error_type(ResponseErrorType::ValidationError)
    .set_error_message(String::from("Could not process request. Make sure your request body is of correct format."))
    .to_response()
    .json_respond()
}

#[catch(default)]
pub fn default_catcher(status: Status, _req: &Request) -> (Status, Json<Response<(), ()>>) {
  ResponseData::<(), ()>::new()
    .set_status(status)
    .set_error_type(ResponseErrorType::UndefinedError)
    .set_error_message(String::from("Could not process request. Contact the administrator."))
    .to_response()
    .json_respond()
}