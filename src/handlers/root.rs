use rocket::{State};
use rocket::response::Redirect;
use crate::responses::*;
use crate::models;
//use crate::guards;
use crate::fairings::database::Pool;
use rocket::http::Status;
use diesel::prelude::*;

pub fn access_link(link_id: String, db: &State<Pool>) -> Result<Redirect, Response<(), ()>> {
  let res_data = ResponseData::<(), ()>::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;

      let conn = &mut *pool;

      match links::table
        .find(&link_id)
        .limit(1)
        .load::<models::Link>(conn) {
          Ok(link) if link.len() > 0 => {
            let target = link[0].target.clone();

            Ok(Redirect::temporary(target))
          },
          Ok(_) => {
            Err(
              errors::AdHocErrors::link_id_not_found(res_data, &link_id)
                .to_response()
            )
          },
          Err(_) => Err(
            res_data
              .set_status(Status::InternalServerError)
              .set_error_type(ResponseErrorType::DatabaseError)
              .set_error_message(String::from("Could not fetch links from the database!"))
              .to_response()
          )
        }
    },
    Err(_) => {
      Err(
        errors::AdHocErrors::database_pool(res_data)
          .to_response()
      )
    }
  }
}