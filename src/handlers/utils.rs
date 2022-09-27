use rocket::{State};
use rocket::http::Status;
use diesel::prelude::*;
use bcrypt;
use url::Url;
use serde::{Serialize};
use nanoid::nanoid;
use crate::fairings::database::Pool;
use crate::responses::*;
use crate::models;
use crate::requests;
use crate::config::Config;

pub fn build_link(base_url: &String, link_id: &String) -> String {
  format!("{}/l/{}", base_url, link_id)
}

pub fn check_id<T: Serialize>(link_id: &String, db: &State<Pool>) -> Result<bool, ResponseData<T>> {
  let response_data = ResponseData::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;
      let conn = &mut *pool;

      match links::table
        .filter(links::link_id.eq(link_id))
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
      errors::Errors::database_pool(response_data)
    )
  }
}

pub fn verify_control_key<S: Serialize>(link_id: &String, control_key: &String, db: &State<Pool>) -> Result<bool, ResponseData<S>> {
  let res_data = ResponseData::new();

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
            errors::Errors::link_id_not_found(res_data, link_id)
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
      errors::Errors::database_pool(res_data)
    )
  }
}

pub fn verify_target(target: &String) -> bool {
  match Url::parse(target) {
    Ok(_) => true,
    Err(_) => false
  }
}

pub fn add_link(link: &requests::NewLink, db: &State<Pool>, config: &State<Config>) -> Result<successes::NewLinkResult, ResponseData<()>> {
  let mut res_data = ResponseData::<()>::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;
      let conn = &mut *pool;

      let base_url = config.base_url.clone();
      let max_id_length = config.max_id_length.clone();
      let max_auto_id_length = config.max_auto_id_length.clone();
      let link_id: Result<String, ()>;
      let control_key = nanoid!(24);
      let target = link.target.clone();

      link_id = match link.link_id.clone() {
        Some(new_link_id) => match check_id(&new_link_id, db) {
          Ok(r) if r == true => {
            if new_link_id.len() > max_id_length {
              res_data = res_data.error(
                Status::BadRequest,
                ResponseErrorType::ValidationError,
                format!("Provided ID is too long!"),
                Some(
                  errors::Errors::LinkIdTooLongError {
                    provided_id_length: new_link_id.len(),
                    max_id_length: max_id_length
                  }
                )
              );
              Err(())
            } else {
              Ok(new_link_id)
            }
          },
          Ok(_) => {
            res_data = errors::Errors::duplicate_id(res_data, &new_link_id);
            Err(())
          },
          Err(res) => { 
            res_data = res;
            Err(())
          }
        },
        None => match links::table
          .select(links::link_id)
          .load::<String>(conn) {
            Ok(link_ids) => {
              let mut new_link_id = nanoid!(max_auto_id_length);
              while link_ids.contains(&new_link_id) {
                new_link_id = nanoid!(max_auto_id_length);
              }
              Ok(new_link_id)
            },
            Err(_) => {
              res_data = res_data
                .error(
                  Status::InternalServerError,
                  ResponseErrorType::DatabaseError,
                  String::from("Could not generate unique link ID due to database error!"),
                  None
                );
              Err(())
            }
          }
      };

      match link_id {
        Ok(link_id) => match verify_target(&target) {
          true => {
            let new_link = successes::NewLinkResult {
              link_id: link_id.clone(),
              target: target,
              control_key: control_key,
              link: build_link(&base_url, &link_id)
            };

            Ok(new_link)
          },
          false => Err(
            errors::Errors::target_invalid(res_data, &target)
          )
        },
        Err(_) => Err(
          res_data
        )
      }
    },
    Err(_) => Err(
      errors::Errors::database_pool(res_data)
    )
  }
}

pub fn get_links(db: &State<Pool>, config: &State<Config>) -> Result<Vec<successes::GetLink>, ResponseData<()>> {
  let res_data = ResponseData::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;
      let conn = &mut *pool;

      let base_url = config.base_url.clone();

      match links::table
        .load::<models::Link>(conn) {
          Ok(links) => {
            let links: Vec<successes::GetLink> = links.iter()
              .map(|r| {
                successes::GetLink {
                  link_id: r.link_id.clone(),
                  target: r.target.clone(),
                  added_at: r.added_at.clone(),
                  visit_count: r.visit_count.clone(),
                  link: build_link(&base_url, &r.link_id)
                }
              })
              .collect();

            Ok(links)
          },
          Err(_) => Err(
            res_data.error(
              Status::InternalServerError,
              ResponseErrorType::DatabaseError,
              String::from("Could not fetch links from the database!"),
              None
            )
          )
        }
    },
    Err(_) => Err(
      errors::Errors::database_pool(res_data)
    )
  }
}

pub fn delete_link(link_id: &String, control_key: &String, db: &State<Pool>) -> Result<(), ResponseData<()>> {
  let res_data = ResponseData::new();

  match check_id(link_id, db) {
    Ok(r) if r == false => match verify_control_key(link_id, control_key, db) {
      Ok(r) if r == true => Ok(()),
      Ok(_) => Err(
        errors::Errors::invalid_control_key(res_data, control_key, link_id)
      ),
      Err(r) => Err(r)
    },
    Ok(_) => Err(
      errors::Errors::link_id_not_found(res_data, link_id)
    ),
    Err(r) => Err(r)
  }
}

pub fn edit_link(link_id: &String, control_key: &String, new_link_id: &Option<String>, new_target: &Option<String>, db: &State<Pool>, config: &State<Config>) -> Result<(String, String), ResponseData<()>> {
  let mut res_data = ResponseData::new();
  let max_id_length = config.max_id_length.clone();

  if Option::is_some(new_link_id) || Option::is_some(new_target) {
    match check_id(link_id, db) {
      Ok(r) if r == false => match verify_control_key(link_id, control_key, db) {
        Ok(r) if r == true => {
          let new_link_id: Result<String, ()> = match new_link_id {
            Some(new_link_id) => match check_id(new_link_id, db) {
              Ok(r) if r == true => {
                if new_link_id.len() <= max_id_length {
                  Ok(new_link_id.clone())
                } else {
                  res_data = res_data.error(
                    Status::BadRequest,
                    ResponseErrorType::ValidationError,
                    String::from("Provided ID is too long!"),
                    Some(
                      errors::Errors::LinkIdTooLongError {
                        provided_id_length: new_link_id.len(),
                        max_id_length: max_id_length
                      }
                    )
                  );
                  Err(())
                }
              },
              Ok(_) => {
                res_data = errors::Errors::duplicate_id(res_data, new_link_id);
                Err(())
              },
              Err(r) => {
                res_data = r;
                Err(())
              }
            },
            None => {
              Ok(link_id.clone())
            }
          };

          let target: Result<String, ()> = match new_target {
            Some(new_target) => match verify_target(new_target) {
              true => Ok(new_target.clone()),
              false => {
                res_data = errors::Errors::target_invalid(res_data, new_target);
                Err(())
              }
            },
            None => match db.get() {
              Ok(mut pool) => {
                use crate::schema::links;
                let conn = &mut *pool;

                match links::table
                  .select(links::target)
                  .filter(links::link_id.eq(link_id))
                  .load::<String>(conn) {
                    Ok(old_target) => {
                      let old_target = old_target[0].clone();

                      Ok(old_target)
                    },
                    Err(_) => {
                      res_data = res_data.error(
                        Status::InternalServerError,
                        ResponseErrorType::DatabaseError,
                        String::from("Could not fetch old target from database."),
                        None
                      );
                      Err(())
                    }
                  }
              },
              Err(_) => {
                res_data = errors::Errors::database_pool(res_data);
                Err(())
              }
            }
          };

          match (new_link_id, target) {
            (Ok(link_id), Ok(target)) => {
              Ok((link_id, target))
            },
            (Ok(_), Err(_)) | (Err(_), Ok(_)) | (Err(_), Err(_)) => {
              Err(res_data)
            }
          }
        },
        Ok(_) => Err(errors::Errors::invalid_control_key(res_data, control_key, link_id)),
        Err(r) => Err(r)
      },
      Ok(_) => Err(errors::Errors::link_id_not_found(res_data, link_id)),
      Err(r) => Err(r)
    }
  } else {
    Err(
      res_data.error(
        Status::BadRequest,
        ResponseErrorType::ValidationError,
        String::from("No editable properties found in request data!"),
        None
      )
    )
  }
}