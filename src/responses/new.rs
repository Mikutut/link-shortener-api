use serde::{Serialize};
use rocket::serde::json::{Json as JsonCons, Value};
use rocket::http::Status;

pub type ResponseResult<S> = (Status, S);
pub type Json = JsonCons<Value>;
pub type JsonErrorResponse<S> = Response<S, Value>;

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum ResponseDataType<S: Serialize> {
  Message(String),
  Value(S)
}

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
  GetLinksError,
  AccessLinkError,
  AddLinkError,
  EditLinkError,
  DeleteLinkError,
  UndefinedError
} 

#[derive(Debug, Serialize, Clone)]
pub struct Response<S: Serialize, E: Serialize> {
  #[serde(skip_serializing)]
  status: Status,
  #[serde(rename = "status")]
  status_string: String,
  #[serde(rename = "code")]
  status_code: u16,
  #[serde(rename = "errorType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  error_type: Option<ResponseErrorType>,
  #[serde(rename = "errorMessage")]
  #[serde(skip_serializing_if = "Option::is_none")]
  error_message: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  data: Option<ResponseDataType<S>>,
  #[serde(rename = "errorData")]
  #[serde(skip_serializing_if = "Option::is_none")]
  error_data: Option<ResponseDataType<E>>
}

impl<S: Serialize, E: Serialize> Response<S, E> {
  pub fn json(self) -> JsonCons<Response<S, E>> {
    JsonCons(self)
  }

  pub fn json_respond(self) -> (Status, JsonCons<Response<S, E>>) {
    (self.status, self.json())
  }
}

pub struct ResponseBuilder<S: Serialize, E: Serialize> {
  status: Status,
  error_type: Option<ResponseErrorType>,
  error_message: Option<String>,
  data: Option<ResponseDataType<S>>,
  error_data: Option<ResponseDataType<E>>
}

impl<S: Serialize, E: Serialize> ResponseBuilder<S, E> {
  pub fn new() -> ResponseBuilder<S, E> {
    ResponseBuilder {
      status: Status::InternalServerError,
      error_type: Some(ResponseErrorType::UndefinedError),
      error_message: Some(String::from("Default response created from ResponseBuilder.")),
      data: None,
      error_data: None
    }
  }

  pub fn success(&mut self, status: Status) {
    if status.code < 400 {
      self.status = status;
      self.error_type = None;
      self.error_message = None;
    } else {
      panic!("error status supplied to success response");
    }
  }
  pub fn error(&mut self, status: Status, error_type: ResponseErrorType, error_message: String) {
    if status.code >= 400 {
      self.status = status;
      self.error_type = Some(error_type);
      self.error_message = Some(error_message);
    } else {
      panic!("info/status/redirect status supplied to error response");
    }
  }

  pub fn data(&mut self, data: ResponseDataType<S>) {
    self.data = Some(data);
  }
  pub fn error_data(&mut self, data: ResponseDataType<E>) {
    self.error_data = Some(data);
  }

  pub fn clear_data(&mut self) {
    self.data = None;
  }
  pub fn clear_error_data(&mut self) {
    self.error_data = None;
  }

  pub fn build(self) -> Response<S, E> {
    let status_str: String = if self.status.code < 400 { String::from("success") } else { String::from("error") };
    let status_code: u16 = self.status.code;

    Response {
      status: self.status,
      status_string: status_str,
      status_code: status_code,
      error_type: self.error_type,
      error_message: self.error_message,
      data: self.data,
      error_data: self.error_data
    }
  }
}