use serde::{Serialize};
use rocket::serde::json::{Json};
use rocket::http::Status;

pub mod errors;
pub mod successes;

#[derive(Debug, Serialize, Clone)]
pub enum ResponseErrorType {
  ValidationError,
  DatabaseError,
  DuplicateIdError,
  InvalidControlKeyError,
  RateLimitedError,
  LinkNotFoundError,
  ControlKeyHashGenerationError,
  ControlKeyHashVerificationError,
  BulkRequestExceedingSizeError,
  BulkRequestError,
  GetLinksError,
  AccessLinkError,
  AddLinkError,
  EditLinkError,
  DeleteLinkError,
  UndefinedError
} 

#[derive(Serialize, Debug, Clone)]
pub enum ResponseType {
  Success,
  Error
}

#[derive(Debug, Serialize, Clone)]
pub struct Response<S: Serialize, E: Serialize> {
  #[serde(skip_serializing)]
  pub status: Status,
  #[serde(rename = "status")]
  pub status_string: String,
  #[serde(rename = "code")]
  pub status_code: u16,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<S>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "errorType")]
  pub error_type: Option<ResponseErrorType>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "errorMessage")]
  pub error_message: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "errorData")]
  pub error_data: Option<E>
}

impl<S: Serialize, E: Serialize> Response<S, E> {
  pub fn json(self) -> Json<Self> {
    Json(self)
  }

  pub fn json_respond(self) -> (Status, Json<Self>) {
    (self.status, self.json())
  }
}

#[derive(Debug, Clone)]
pub struct ResponseData<S: Serialize, E: Serialize> {
  status: Status,
  status_type: ResponseType,
  data: Option<S>,
  error_type: Option<ResponseErrorType>,
  error_message: Option<String>,
  error_data: Option<E>
}

impl<S: Serialize, E: Serialize> ResponseData<S, E> {
  pub fn new() -> Self {
    ResponseData {
      status: Status::InternalServerError,
      status_type: ResponseType::Error,
      data: None,
      error_type: Some(ResponseErrorType::UndefinedError),
      error_message: Some(String::from("Default response.")),
      error_data: None
    }
  }

  pub fn transform<T>(self, data: T) -> ResponseData<T, E>
  where
    T: Serialize 
  {
    ResponseData {
      status: self.status,
      status_type: self.status_type,
      data: Some(data),
      error_type: self.error_type,
      error_message: self.error_message,
      error_data: self.error_data
    }
  }

  pub fn transform_error<T>(self, data: T) -> ResponseData<S, T>
  where
    T: Serialize
  {
    ResponseData {
      status: self.status,
      status_type: self.status_type,
      data: self.data,
      error_type: self.error_type,
      error_message: self.error_message,
      error_data: Some(data)
    }
  }

  pub fn get_status(&self) -> &Status {
    &self.status
  }
  pub fn get_status_type(&self) -> &ResponseType {
    &self.status_type
  }
  pub fn get_data(&self) -> Option<&S> {
    match &self.data {
      Some(data) => Some(data),
      None => None
    }
  }
  pub fn get_error_type(&self) -> Option<&ResponseErrorType> {
    match &self.error_type {
      Some(error_type) => Some(error_type),
      None => None
    }
  }
  pub fn get_error_message(&self) -> Option<&String> {
    match &self.error_message {
      Some(error_message) => Some(error_message),
      None => None
    }
  }
  pub fn get_error_data(&self) -> Option<&E> {
    match &self.error_data {
      Some(error_data) => Some(error_data),
      None => None
    }
  }

  pub fn set_status(mut self, status: Status) -> Self {
    self.status = status;
    self
  }
  pub fn set_type(mut self, response_type: ResponseType) -> Self {
    self.status_type = response_type;
    self
  }
  pub fn set_data(mut self, data: S) -> Self {
    self.data = Some(data);
    self
  }
  pub fn set_error_type(mut self, error_type: ResponseErrorType) -> Self {
    self.error_type = Some(error_type);
    self
  }
  pub fn set_error_message(mut self, error_message: String) -> Self {
    self.error_message = Some(error_message);
    self
  }
  pub fn set_error_data(mut self, data: E) -> Self {
    self.error_data = Some(data);
    self
  }

  pub fn clear_data(mut self) -> Self {
    self.data = None;
    self
  }
  pub fn clear_error_type(mut self) -> Self {
    self.error_type = None;
    self
  }
  pub fn clear_error_message(mut self) -> Self {
    self.error_message = None;
    self
  }
  pub fn clear_error_data(mut self) -> Self {
    self.error_data = None;
    self
  }

  pub fn success(mut self, status: Status, data: Option<S>) -> Self {
    self = self
      .clear_error_type()
      .clear_error_message()
      .clear_error_data();

    self.status = status;
    self.data = data;

    self
  }
  pub fn error(mut self, status: Status, error_type: ResponseErrorType, error_message: String, error_data: Option<E>) -> Self {
    self = self
      .clear_data();

    self.status = status;
    self.error_type = Some(error_type);
    self.error_message = Some(error_message);
    self.error_data = error_data;

    self
  }

  pub fn to_response(self) -> Response<S, E> {
    let code = self.status.code;

    Response {
      status: self.status,
      status_string: String::from(if code < 400 { "success" } else { "error" }),
      status_code: code,
      data: self.data,
      error_type: self.error_type,
      error_message: self.error_message,
      error_data: self.error_data
    }
  }
}