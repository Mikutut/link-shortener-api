use rocket::{State};
use rocket::http::Status;
use rocket::serde::json::{Value as JsonValue};
use diesel::prelude::*;
use bcrypt;
use crate::fairings::database::Pool;
use crate::responses::*;
use crate::models;

pub fn check_id(link_id: String, db: &State<Pool>) -> Result<bool, ResponseData<(), JsonValue>> {
  let response_data = ResponseData::<(), JsonValue>::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;
      let conn = &mut *pool;

      match links::table
        .filter(links::link_id.eq(&link_id))
        .count()
        .get_result::<i64>(conn) {
          Ok(c) if c == 0 => Ok(true),
          Ok(_) => Ok(false),
          Err(_) => Err(
            response_data
              .set_status(Status::InternalServerError)
              .set_error_type(ResponseErrorType::DatabaseError)
              .set_error_message(format!("Could not verify presence of link with ID '{}'", link_id))
          )
        }
    },
    Err(_) => Err(
      errors::AdHocErrors::database_pool(response_data)
    )
  }
}

pub fn verify_control_key(link_id: &String, control_key: &String, db: &State<Pool>) -> Result<bool, ResponseData<(), ()>> {
  let res_data = ResponseData::<(), ()>::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;
      let conn = &mut *pool;

      match links::table
        .filter(links::link_id.eq(link_id))
        .load::<models::Link>(conn) {
          Ok(link) if link.len() > 0 => {
            let link = link[0].clone();
            let db_control_key = link.control_key;

            match bcrypt::verify(control_key, &db_control_key) {
              Ok(r) => Ok(r),
              Err(_) => Err(
                res_data
                  .error(
                    Status::InternalServerError, 
                    ResponseErrorType::ControlKeyHashVerificationError, 
                    String::from("Could not verify validity of control key!"), 
                    None
                  )
              )
            }
          },
          Ok(_) => Err(
            errors::AdHocErrors::link_id_not_found(res_data, link_id)
          ),
          Err(_) => Err(
            res_data
              .error(
                Status::InternalServerError, 
                ResponseErrorType::DatabaseError, 
                String::from("Could not get links from database!"), 
                None
              )
          )
        }
    },
    Err(_) => Err(
      errors::AdHocErrors::database_pool(res_data)
    )
  }
}