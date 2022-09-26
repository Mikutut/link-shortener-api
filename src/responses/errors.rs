use std::marker::PhantomData;
use serde::{Serialize};
use super::*;
use rocket::http::Status;

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum Errors<E: Serialize> {
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
    request_error_data: Option<E>
  },
  LinkIdTooLongError {
    #[serde(rename = "providedIdLength")]
    provided_id_length: usize,
    #[serde(rename = "maxIdLength")]
    max_id_length: usize
  },
  CustomError(E),
  NoError
}

impl<E: Serialize> Errors<E> {
  pub fn transform<F: Serialize>(self, new_error: Errors<F>) -> Errors<F> {
    new_error
  }
}

pub struct AdHocErrors<S: Serialize, E: Serialize> {
  _one: PhantomData<S>,
  _two: PhantomData<E>
}

impl<S: Serialize, E: Serialize> AdHocErrors<S, E> {
  pub fn invalid_control_key(mut response_data: ResponseData<S, E>, control_key: &String, link_id: &String) -> ResponseData<S, E> {
    response_data = response_data
      .set_status(Status::Unauthorized)
      .set_error_type(ResponseErrorType::InvalidControlKeyError)
      .set_error_message(format!("'{}' is not a valid control key for link with ID '{}'!", control_key, link_id));

    response_data
  }
  pub fn database_pool(mut response_data: ResponseData<S, E>) -> ResponseData<S, E> {
    response_data = response_data
      .set_status(Status::InternalServerError)
      .set_error_type(ResponseErrorType::DatabaseError)
      .set_error_message(String::from("Could not get database pool!"));

    response_data
  }
  pub fn link_id_not_found(mut response_data: ResponseData<S, E>, link_id: &String) -> ResponseData<S, E> {
    response_data = response_data
      .set_status(Status::NotFound)
      .set_error_type(ResponseErrorType::LinkNotFoundError)
      .set_error_message(format!("Link with ID '{}' not found!", link_id));

    response_data
  }
  pub fn target_invalid(mut response_data: ResponseData<S, E>, target: &String) -> ResponseData<S, E> {
    response_data = response_data
      .set_status(Status::BadRequest)
      .set_error_type(ResponseErrorType::ValidationError)
      .set_error_message(format!("Target '{}' is not a valid URL!", target));

    response_data
  }
  pub fn duplicate_id(mut response_data: ResponseData<S, E>, link_id: &String) -> ResponseData<S, E> {
    response_data = response_data.error(
      Status::Conflict,
      ResponseErrorType::DuplicateIdError,
      format!("Link with ID '{}' already exists!", link_id),
      None
    );

    response_data
  }
}