use serde::{Serialize};
use rocket::serde::json::{Json, Value};
use rocket::http::Status;

pub mod errors;

pub type ResponseResult<S> = (Status, S);
pub type JsonErrorResponse<S> = Response<S, Value>;
pub type EmptyResponse<E> = Response<(), E>;

#[derive(Debug, Clone)]
enum ResponseStatusType {
  Success,
  Error
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum ResponseDataType<S: Serialize> {
  Message(String),
  Value(S)
}

impl<S: Serialize> ResponseDataType<S> {
  pub fn extract_value(&self) -> Option<&S> {
    match self {
      ResponseDataType::Message(_) => None,
      ResponseDataType::Value(v) => Some(v)
    }
  }
  pub fn extract_message(&self) -> Option<&String> {
    match self {
      ResponseDataType::Message(msg) => Some(msg),
      ResponseDataType::Value(_) => None
    }
  }
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
  #[serde(skip_serializing)]
  status_type: ResponseStatusType,
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
  pub fn is_success(res: &Response<S, E>) -> bool {
    match &res.status_type {
      ResponseStatusType::Success => true,
      ResponseStatusType::Error => false
    }
  }
  pub fn is_error(res: &Response<S, E>) -> bool {
    match &res.status_type {
      ResponseStatusType::Success => false,
      ResponseStatusType::Error => true
    }
  }

  pub fn json(self) -> Json<Response<S, E>> {
    Json(self)
  }

  pub fn json_respond(self) -> (Status, Json<Response<S, E>>) {
    (self.status, self.json())
  }
}

pub struct ResponseBuilder<S: Serialize, E: Serialize> {
  status: Status,
  status_type: ResponseStatusType,
  error_type: Option<ResponseErrorType>,
  error_message: Option<String>,
  data: Option<ResponseDataType<S>>,
  error_data: Option<ResponseDataType<E>>
}

impl<S: Serialize, E: Serialize> ResponseBuilder<S, E> {
  pub fn new() -> ResponseBuilder<S, E> {
    ResponseBuilder {
      status: Status::InternalServerError,
      status_type: ResponseStatusType::Error,
      error_type: Some(ResponseErrorType::UndefinedError),
      error_message: Some(String::from("Default response created from ResponseBuilder.")),
      data: None,
      error_data: None
    }
  }

  pub fn success(&mut self, status: Status) -> &mut Self {
    if status.code < 400 {
      self.status = status;
      self.status_type = ResponseStatusType::Success;
      self.error_type = None;
      self.error_message = None;

      self
    } else {
      panic!("error status supplied to success response");
    }
  }
  pub fn error(&mut self, status: Status, error_type: ResponseErrorType, error_message: String) -> &mut Self {
    if status.code >= 400 {
      self.status = status;
      self.status_type = ResponseStatusType::Error;
      self.error_type = Some(error_type);
      self.error_message = Some(error_message);

      self
    } else {
      panic!("info/status/redirect status supplied to error response");
    }
  }

  pub fn data(&mut self, data: ResponseDataType<S>) -> &mut Self {
    self.data = Some(data);
    self
  }
  pub fn error_data(&mut self, data: ResponseDataType<E>) -> &mut Self {
    self.error_data = Some(data);
    self
  }

  pub fn get_data(&self) -> &Option<ResponseDataType<S>> {
    &self.data
  }
  pub fn clear_data(&mut self) -> &mut Self {
    self.data = None;
    self
  }

  pub fn get_error_data(&self) -> &Option<ResponseDataType<E>> {
    &self.error_data
  }
  pub fn clear_error_data(&mut self) -> &mut Self {
    self.error_data = None;
    self
  }

  pub fn build(self) -> Response<S, E> {
    let status_str: String = if self.status.code < 400 { String::from("success") } else { String::from("error") };
    let status_code: u16 = self.status.code;

    Response {
      status: self.status,
      status_string: status_str,
      status_code: status_code,
      status_type: self.status_type,
      error_type: self.error_type,
      error_message: self.error_message,
      data: self.data,
      error_data: self.error_data
    }
  }

  pub fn is_success(response_builder: &ResponseBuilder<S, E>) -> bool {
    match &response_builder.status_type {
      ResponseStatusType::Success => true,
      ResponseStatusType::Error => false
    }
  }
  pub fn is_error(response_builder: &ResponseBuilder<S, E>) -> bool {
    match &response_builder.status_type {
      ResponseStatusType::Success => false,
      ResponseStatusType::Error => true
    }
  }
}