use std::marker::PhantomData;
use serde::{Serialize};
use super::*;
use rocket::http::Status;

#[derive(Debug, Serialize, Clone)]
pub struct BulkRequestError<E: Serialize> {
  #[serde(rename = "requestNumber")]
  pub request_number: u32,
  #[serde(rename = "requestErrorType")]
  pub request_error_type: ResponseErrorType,
  #[serde(rename = "requestErrorMessage")]
  pub request_error_message: String,
  #[serde(rename = "requestErrorData")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub request_error_data: Option<E>
}

pub struct AdHocErrors<S: Serialize, E: Serialize> {
  _one: PhantomData<S>,
  _two: PhantomData<E>
}

impl<S: Serialize, E: Serialize> AdHocErrors<S, E> {
  pub fn invalid_control_key(response_builder: &mut ResponseBuilder<S, E>, control_key: &String, link_id: &String) {
    response_builder.error(
      Status::Unauthorized,
      ResponseErrorType::InvalidControlKeyError,
      format!("'{}' is not a valid control key for link with ID '{}'!",
        control_key,
        link_id
      )
    );
  }
  pub fn database_pool(response_builder: &mut ResponseBuilder<S, E>) {
    response_builder.error(
      Status::InternalServerError,
      ResponseErrorType::DatabaseError,
      String::from("Could not get database pool!")
    );
  }
  pub fn link_id_not_found(response_builder: &mut ResponseBuilder<S, E>, link_id: &String) {
    response_builder.error(
      Status::NotFound,
      ResponseErrorType::LinkNotFoundError,
      format!("Link with ID '{}' not found!", link_id)
    );
  }
  pub fn target_invalid(response_builder: &mut ResponseBuilder<S, E>, target: &String) {
    response_builder.error(
      Status::BadRequest,
      ResponseErrorType::ValidationError,
      format!("Target '{}' is not a valid URL!", target)
    );
  }
}