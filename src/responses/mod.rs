//TODO: Replace old response system with new one (in new submodule)

use serde::{Serialize};
use rocket::serde::json::Json;
use rocket::http::Status;

pub mod new;
pub mod errors;

pub type Success<S> = (Status, Json<S>);
pub type Error = (Status, Json<ResponseError>);
pub type Result<S> = core::result::Result<Success<S>, Error>;

#[derive(Debug, Serialize)]
pub enum ResponseErrorType {
  ValidationError,
  DatabaseError,
  DuplicateIdError,
  InvalidControlKeyError,
  RateLimitedError,
  LinkNotFoundError,
  ControlKeyHashGenerationError,
  ControlKeyHashVerificationError,
  GetLinksError,
  AccessLinkError,
  AddLinkError,
  EditLinkError,
  DeleteLinkError,
  UndefinedError
} 

#[deprecated]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseError {
  pub error_type: ResponseErrorType,
  pub error_message: String
}

#[deprecated]
impl ResponseError {
  pub fn new(error_type: ResponseErrorType, error_message: String) -> Self {
    ResponseError {
      error_type: error_type,
      error_message: error_message
    }
  }

  pub fn to_json(self) -> Json<Self> {
    Json(self)
  }
}