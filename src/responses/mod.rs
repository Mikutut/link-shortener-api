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
pub struct Response<S: Serialize> {
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
  pub error_data: Option<errors::Errors>
}

impl<S: Serialize> Response<S> {
  pub fn json(self) -> Json<Self> {
    Json(self)
  }

  pub fn json_respond(self) -> (Status, Json<Self>) {
    (self.status, self.json())
  }
}

#[derive(Debug, Clone)]
pub struct ResponseData<S: Serialize> {
  status: Status,
  status_type: ResponseType,
  data: Option<S>,
  error_type: Option<ResponseErrorType>,
  error_message: Option<String>,
  error_data: Option<errors::Errors>
}

impl<S: Serialize> ResponseData<S> {
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

  pub fn transform<T>(self, data: Option<T>) -> ResponseData<T>
  where
    T: Serialize 
  {
    ResponseData {
      status: self.status,
      status_type: self.status_type,
      data: data,
      error_type: self.error_type,
      error_message: self.error_message,
      error_data: self.error_data
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
  pub fn get_error_data(&self) -> Option<&errors::Errors> {
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
  pub fn set_error_data(mut self, data: errors::Errors) -> Self {
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

  pub fn clone_status(&self) -> Status {
    self.status.clone()
  }
  pub fn clone_error_type(&self) -> Option<ResponseErrorType> {
    self.error_type.clone()
  }
  pub fn clone_error_message(&self) -> Option<String> {
    self.error_message.clone()
  }
  pub fn clone_error_data(&self) -> Option<errors::Errors> {
    match &self.error_data {
      Some(data) => Some(data.clone()),
      None => None
    }
  }

  pub fn success(mut self, status: Status, data: Option<S>) -> Self {
    self = self
      .clear_error_type()
      .clear_error_message()
      .clear_error_data();

    self.status = status;
    self.status_type = ResponseType::Success;
    self.data = data;

    self
  }
  pub fn error(mut self, status: Status, error_type: ResponseErrorType, error_message: String, error_data: Option<errors::Errors>) -> Self {
    self = self
      .clear_data();

    self.status = status;
    self.status_type = ResponseType::Error;
    self.error_type = Some(error_type);
    self.error_message = Some(error_message);
    self.error_data = error_data;

    self
  }

  pub fn to_response(self) -> Response<S> {
    let code = self.status.code;

    Response {
      status: self.status,
      status_string: String::from(if let ResponseType::Success = self.status_type { "success" } else { "error" }),
      status_code: code,
      data: self.data,
      error_type: self.error_type,
      error_message: self.error_message,
      error_data: self.error_data
    }
  }
}

impl<S: Serialize + Clone> ResponseData<S> {
  pub fn clone(&self) -> ResponseData<S> {
    ResponseData {
      status: self.status.clone(),
      status_type: self.status_type.clone(),
      data: self.data.clone(),
      error_type: self.error_type.clone(),
      error_message: self.error_message.clone(),
      error_data: self.error_data.clone()
    }
  }
  pub fn clone_data(&self) -> Option<S> {
    match &self.data {
      Some(data) => Some(data.clone()),
      None => None
    }
  }
}
