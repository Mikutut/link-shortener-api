use rocket::{self, get, State, response::Redirect};
use rocket::serde::{json::{Json, Value as JsonValue}};
use rocket::http::Status;
use crate::fairings::database::Pool;
use crate::{guards, responses::*, handlers};

#[get("/check-id/<link_id>")]
pub fn get_check_id(link_id: String, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> (Status, Json<Response<bool, JsonValue>>) {
  let res_data = ResponseData::<bool, JsonValue>::new();

  match handlers::utils::check_id(link_id, db) {
    Ok(r) => {
      res_data
        .success(
          Status::Ok,
          Some(r)
        )
        .to_response()
        .json_respond()
    },
    Err(r) => {
      r.transform::<bool>(false)
        .clear_data()
        .to_response()
        .json_respond()
    }
  }
}

#[get("/<link_id>")]
pub fn get_access_link(link_id: String, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> Result<Redirect, (Status, Json<Response<(), ()>>)> {
  match handlers::root::access_link(link_id, db) {
    Ok(redirect) => Ok(redirect),
    Err(response) => Err(response.json_respond())
  }
}