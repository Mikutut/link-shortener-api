use serde::{Serialize};
use super::*;

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