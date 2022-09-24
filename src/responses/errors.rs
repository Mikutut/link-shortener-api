use serde::{Serialize};
use super::*;

#[derive(Debug, Serialize, Clone)]
pub struct BulkRequestError<E: Serialize> {
  #[serde(rename = "requestNumber")]
  pub request_number: i32,
  #[serde(rename = "requestError")]
  pub request_error: ResponseErrorType,
  #[serde(rename = "requestErrorMessage")]
  pub request_error_message: String,
  #[serde(rename = "requestErrorData")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub request_error_data: Option<E>
}