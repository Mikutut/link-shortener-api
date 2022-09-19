use rocket::{catch};

#[catch(422)]
pub fn invalid_request_data() -> &'static str {
  "Could not process request. Make sure your request body is of correct format."
}