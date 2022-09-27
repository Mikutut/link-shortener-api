use rocket::{State};
use rocket::response::Redirect;
use rocket::http::Status;
use rocket::serde::json::{Json};
use diesel::prelude::*;
use crate::responses::*;
use crate::models;
//use crate::guards;
use crate::fairings::database::Pool;
use crate::config::Config;
use crate::requests;

pub fn access_link(link_id: String, db: &State<Pool>) -> Result<Redirect, Response<()>> {
  let res_data = ResponseData::<()>::new();

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
              errors::Errors::link_id_not_found(res_data, &link_id)
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
        errors::Errors::database_pool(res_data)
          .to_response()
      )
    }
  }
}

pub fn get_links(db: &State<Pool>, config: &State<Config>) -> (Status, Json<Response<Vec<successes::GetLink>>>) {
  let res_data = ResponseData::<Vec<successes::GetLink>>::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;
      let conn = &mut *pool;

      let base_url = config.base_url.clone();

      match links::table
        .order(links::added_at.desc())
        .load::<models::Link>(conn) {
          Ok(links) => {
            let data = links.iter()
              .map(|r| {
                successes::GetLink {
                  link_id: r.link_id.clone(),
                  target: r.target.clone(),
                  added_at: r.added_at.clone(),
                  visit_count: r.visit_count.clone(),
                  link: format!("{}/{}", base_url, r.link_id.clone())
                }
              })
              .collect::<Vec<successes::GetLink>>();

            res_data.success(
              Status::Ok,
              Some(data)
            )
              .to_response()
              .json_respond()
          },
          Err(_) => {
            res_data
              .error(
                Status::InternalServerError,
                ResponseErrorType::DatabaseError,
                String::from("Could not fetch links from database!"),
                None
              )
              .to_response()
              .json_respond()
          }
        }
    },
    Err(_) => {
      errors::Errors::database_pool(res_data)
        .to_response()
        .json_respond()
    }
  }
}

pub fn add_link(link: &requests::NewLink, db: &State<Pool>, config: &State<Config>) -> (Status, Json<Response<successes::NewLinkResult>>) {
  let res_data = ResponseData::new();

  match super::utils::add_link(link, db, config) {
    Ok(new_link) => {
      let link_id = new_link.link_id.clone();
      let control_key = new_link.control_key.clone();
      let target = new_link.target.clone();

      match bcrypt::hash(&control_key, bcrypt::DEFAULT_COST) {
        Ok(hash) => match db.get() {
          Ok(mut pool) => {
            use crate::schema::links;
            let conn = &mut *pool;

            let new_link_db = models::NewLink {
              link_id: link_id.clone(),
              control_key: hash,
              target: target.clone()
            };

            match diesel::insert_into(links::table)
              .values(new_link_db)
              .execute(conn) {
                Ok(_) => {
                  res_data.success(
                    Status::Ok,
                    Some(new_link)
                  )
                    .to_response()
                    .json_respond()
                },
                Err(_) => {
                  res_data.error(
                    Status::InternalServerError,
                    ResponseErrorType::DatabaseError,
                    format!("Could not add link with ID '{}' to database!", link_id),
                    None
                  )
                    .to_response()
                    .json_respond()
                }
              }
          },
          Err(_) => errors::Errors::database_pool(res_data).to_response().json_respond()
        },
        Err(_) => {
          res_data.error(
            Status::InternalServerError,
            ResponseErrorType::ControlKeyHashGenerationError,
            format!("Could not generate bcrypt hash for control key '{}'!", control_key),
            None
          )
            .to_response()
            .json_respond()
        }
      }
    },
    Err(res) => {
      res
        .transform(None)
        .to_response()
        .json_respond()
    }
  }
}

pub fn delete_link(link_id: &String, control_key: &String, db: &State<Pool>) -> Result<(), ResponseData<()>> {
  let res_data = ResponseData::new();

  match db.get() {
    Ok(mut pool) => match super::utils::delete_link(link_id, control_key, db) {
      Ok(()) => {
        use crate::schema::links;
        let conn = &mut *pool;

        match diesel::delete(links::table)
          .filter(links::link_id.eq(link_id))
          .execute(conn) {
            Ok(_) => Ok(()),
            Err(_) => Err(
              res_data.error(
                Status::InternalServerError,
                ResponseErrorType::DatabaseError,
                format!("Could not delete link with ID '{}' from database!", link_id),
                None
              )
            )
          }
      },
      Err(r) => Err(r)
    },
    Err(_) => Err(errors::Errors::database_pool(res_data))
  }
}

pub fn edit_link(link_id: &String, control_key: &String, new_link_id: &Option<String>, new_target: &Option<String>, db: &State<Pool>, config: &State<Config>) -> Result<successes::EditLinkResult, ResponseData<()>> {
  let res_data = ResponseData::new();
  let base_url = config.base_url.clone();

  match super::utils::edit_link(link_id, control_key, new_link_id, new_target, db, config) {
    Ok((new_link_id, new_target)) => match db.get() {
      Ok(mut pool) => {
        use crate::schema::links;
        let conn = &mut *pool;

        match diesel::update(links::table)
          .set((links::link_id.eq(&new_link_id), links::target.eq(&new_target)))
          .filter(links::link_id.eq(link_id))
          .execute(conn) {
            Ok(_) => {
              Ok(successes::EditLinkResult {
                link_id: new_link_id.clone(),
                target: new_target.clone(),
                link: format!("{}/{}", base_url, new_link_id)
              })
            },
            Err(_) => Err(
              res_data.error(
                Status::InternalServerError,
                ResponseErrorType::DatabaseError,
                format!("Could not update link with ID '{}' due to database error!", link_id),
                None
              )
            )
          }
      },
      Err(_) => Err(errors::Errors::database_pool(res_data))
    },
    Err(r) => Err(r)
  }
}