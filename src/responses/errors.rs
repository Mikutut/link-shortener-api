use serde::{Serialize};
use super::*;
use rocket::http::Status;
use std::boxed::Box;

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum Errors {
  RateLimitedError {
    #[serde(rename = "maxRequests")]
    max_requests: i64,
    #[serde(rename = "timeWindow")]
    time_window: i64,
    cooldown: i64
  },
  BulkRequestError {
    #[serde(rename = "requestNumber")]
    request_number: u32,
    #[serde(rename = "requestErrorType")]
    request_error_type: ResponseErrorType,
    #[serde(rename = "requestErrorMessage")]
    request_error_message: String,
    #[serde(rename = "requestErrorData")]
    #[serde(skip_serializing_if = "Option::is_none")]
    request_error_data: Option<Box<Errors>>
  },
  LinkIdTooLongError {
    #[serde(rename = "providedIdLength")]
    provided_id_length: usize,
    #[serde(rename = "maxIdLength")]
    max_id_length: usize
  },
  NoError
}

impl Errors {
  pub fn invalid_control_key<S: Serialize>(mut response_data: ResponseData<S>, control_key: &String, link_id: &String) -> ResponseData<S> {
    response_data = response_data
      .set_status(Status::Unauthorized)
      .set_error_type(ResponseErrorType::InvalidControlKeyError)
      .set_error_message(format!("'{}' is not a valid control key for link with ID '{}'!", control_key, link_id));

    response_data
  }
  pub fn database_pool<S: Serialize>(mut response_data: ResponseData<S>) -> ResponseData<S> {
    response_data = response_data
      .set_status(Status::InternalServerError)
      .set_error_type(ResponseErrorType::DatabaseError)
      .set_error_message(String::from("Could not get database pool!"));

    response_data
  }
  pub fn link_id_not_found<S: Serialize>(mut response_data: ResponseData<S>, link_id: &String) -> ResponseData<S> {
    response_data = response_data
      .set_status(Status::NotFound)
      .set_error_type(ResponseErrorType::LinkNotFoundError)
      .set_error_message(format!("Link with ID '{}' not found!", link_id));

    response_data
  }
  pub fn duplicate_id<S: Serialize>(mut response_data: ResponseData<S>, link_id: &String) -> ResponseData<S> {
    response_data = response_data.error(
      Status::Conflict,
      ResponseErrorType::DuplicateIdError,
      format!("Link with ID '{}' already exists!", link_id),
      None
    );

    response_data
  }
  pub fn target_invalid<S: Serialize>(mut response_data: ResponseData<S>, target: &String) -> ResponseData<S> {
    response_data = response_data
      .set_status(Status::BadRequest)
      .set_error_type(ResponseErrorType::ValidationError)
      .set_error_message(format!("Target '{}' is not a valid URL!", target));

    response_data
  }
}