use crate::models;
use rocket::response::{Redirect};
use rocket::{self, get, post, delete, patch, State};
use rocket::http::Status;
use rocket::serde::{json::{Json, Value}};
use nanoid::{nanoid};
use crate::fairings::database::Pool;
use crate::{guards, responses::*, config::Config};
use bcrypt;
use url::{Url};

use diesel::QueryDsl;
use diesel::prelude::*;

#[get("/<link_id>")]
pub fn access_link(link_id: String, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> Result<Redirect, ResponseResult<Json<JsonErrorResponse<()>>>> {
  let mut response_builder = ResponseBuilder::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;

      let conn = &mut *pool;

      match links::table
        .find(link_id.clone())
        .limit(1)
        .load::<models::Link>(conn) {
          Ok(link) if link.len() > 0 => {
            let target = link[0].clone().target;

            return Ok(Redirect::temporary(target));
          },
          Ok(_) => {
            response_builder.error(
              Status::NotFound,
              ResponseErrorType::LinkNotFoundError,
              format!("Link with ID '{}' not found!", link_id)
            );
          },
          Err(_) => {
            response_builder.error(
              Status::InternalServerError,
              ResponseErrorType::DatabaseError,
              String::from("Could not fetch links from the database!")
            );
          }
        }
    },
    Err(_) => {
      response_builder.error(
        Status::InternalServerError,
        ResponseErrorType::DatabaseError,
        String::from("Could not get database pool!")
      );
    }
  }

  Err(response_builder.build().json_respond())
}

#[get("/get-links")]
pub fn get_links(db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> ResponseResult<Json<JsonErrorResponse<Vec<models::db_less::GetLink>>>> {
  let mut response_builder = ResponseBuilder::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;

      let conn = &mut *pool;
      let base_url = config.base_url.clone();

      match links::table
        .order(links::added_at.desc())
        .load::<models::Link>(conn) {
          Ok(links) => {
            let data = ResponseDataType::Value(
              links.iter()
                .map(|r| {
                  models::db_less::GetLink {
                    link_id: r.link_id.clone(),
                    target: r.target.clone(),
                    added_at: r.added_at.clone(),
                    visit_count: r.visit_count.clone(),
                    link: format!("{}/{}", base_url, r.link_id.clone())
                  }
                })
                .collect()
            );

            response_builder.success(Status::Ok);
            response_builder.data(data);
          },
          Err(_) => {
            response_builder.error(
              Status::InternalServerError,
              ResponseErrorType::DatabaseError,
              String::from("Could not fetch links from database!")
            );
          }
        }
    },
    Err(_) => {
      response_builder.error(
        Status::InternalServerError, 
        ResponseErrorType::DatabaseError, 
        "Could not get database pool!".into()
      );
    }
  }

  response_builder.build().json_respond()
}

#[post("/add-link", data = "<link>", rank = 1)]
pub fn add_link(link: Json<models::db_less::NewLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> ResponseResult<Json<JsonErrorResponse<models::db_less::NewLinkResult>>> {
  let mut response_builder = ResponseBuilder::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;
      let conn = &mut *pool;

      let link_id = link.link_id.clone();
      let new_link_id: Result<String, ()>;
      let new_control_key = nanoid!(24);
      let new_target = link.target.clone();

      let base_url = config.base_url.clone();
      let max_id_length = config.max_id_length.clone();
      let max_auto_id_length = config.max_auto_id_length.clone();

      match bcrypt::hash(new_control_key.clone(), bcrypt::DEFAULT_COST) {
        Ok(new_control_key_hash) => {
          let result = links::table
            .select(links::link_id)
            .load::<String>(conn);

          if let Ok(results) = result {
            new_link_id = match link_id {
              Some(link_id) if link_id.len() <= max_id_length => {
                match links::table
                  .filter(links::link_id.eq(link_id.clone()))
                  .count()
                  .get_result::<i64>(conn) {
                    Ok(c) if c == 0 => {
                      Ok(link_id)
                    },
                    Ok(_) => {
                      response_builder.error(
                        Status::Conflict,
                        ResponseErrorType::DuplicateIdError,
                        format!("ID '{}' is already in use!", link_id)
                      );
                      Err(())
                    },
                    Err(_) => {
                      response_builder.error(
                        Status::InternalServerError,
                        ResponseErrorType::DatabaseError,
                        String::from("Could not verify uniqueness of new ID!")
                      );
                      Err(())
                    }
                  }
              },
              Some(link_id) => {
                response_builder.error(
                  Status::BadRequest,
                  ResponseErrorType::ValidationError,
                  format!("ID '{}' is too long! (received length: {}, max length: {})",
                    link_id.clone(),
                    link_id.len(),
                    max_id_length
                  )
                );
                Err(())
              }
              None => {
                let mut temp_id = nanoid!(max_auto_id_length);
                while results.contains(&temp_id) {
                  temp_id = nanoid!(max_auto_id_length);
                }

                Ok(temp_id)
              }
            };

            match new_link_id {
              Ok(new_link_id) => {
                let new_link = models::NewLink {
                  link_id: new_link_id.clone(),
                  target: new_target,
                  control_key: new_control_key_hash
                };

                let result = diesel::insert_into(links::table)
                  .values(new_link)
                  .execute(conn);

                if let Ok(_) = result {
                  let result = links::table
                    .find(new_link_id)
                    .load::<models::Link>(conn);

                  if let Ok(new_link) = result {
                    let new_link = new_link[0].clone();

                    response_builder.success(Status::Ok);
                    response_builder.data(
                      ResponseDataType::Value(models::db_less::NewLinkResult {
                        link_id: new_link.link_id.clone(),
                        target: new_link.target,
                        control_key: new_control_key,
                        link: format!("{}/{}", base_url, new_link.link_id)
                      })
                    );              
                  } else {
                    response_builder.error(
                      Status::InternalServerError,
                      ResponseErrorType::DatabaseError,
                      String::from("New link was added, but server could not retrieve required data.")
                    );
                  }
                } else {
                  response_builder.error(
                    Status::InternalServerError,
                    ResponseErrorType::DatabaseError,
                    String::from("Could not add link to database!")
                  );
                }
              },
              Err(_) => {}
            }
          } else {
            response_builder.error(
              Status::InternalServerError,
              ResponseErrorType::DatabaseError,
              String::from("Could not send database query!")
            );
          }
        },
        Err(_) => {
          response_builder.error(
            Status::InternalServerError,
            ResponseErrorType::ControlKeyHashGenerationError,
            String::from("Could not generate control key hash!")
          );
        }
      }
    },
    Err(_) => {
      response_builder.error(
        Status::InternalServerError, 
        ResponseErrorType::DatabaseError, 
        String::from("Could not get database pool!")
      );
    }
  }

  response_builder.build().json_respond()
}

#[delete("/delete-link", data = "<link>")]
pub fn delete_link(link: Json<models::db_less::DeleteLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> ResponseResult<Json<JsonErrorResponse<()>>> {
  let mut response_builder = ResponseBuilder::new();

  if let Ok(mut pool) = db.get() {
    let conn = &mut *pool;
    use crate::schema::links;

    let link_id = link.0.link_id;
    let control_key = link.0.control_key;

    match links::table
      .filter(links::link_id.eq(link_id.clone()))
      .load::<models::Link>(conn) {
        Ok(link) if link.len() == 1 => {
          let link = link[0].clone();

          match bcrypt::verify(control_key.clone(), &link.control_key) {
            Ok(bcrypt_result) if bcrypt_result == true => {
              match diesel::delete(links::table)
                .filter(links::link_id.eq(link_id.clone()))
                .execute(conn) {
                  Ok(_) => {
                    response_builder.success(
                      Status::Ok
                    );
                  },
                  Err(_) => {
                    response_builder.error(
                      Status::InternalServerError,
                      ResponseErrorType::DatabaseError,
                      format!("Could not delete link with ID '{}' from database!", link_id)
                    );
                  }
                }
            },
            _ => {
              response_builder.error(
                Status::Unauthorized,
                ResponseErrorType::InvalidControlKeyError,
                format!("'{}' is not a valid control key for '{}' link!", control_key, link_id)
              );
            }
          }
        },
        _ => {
          response_builder.error(
            Status::NotFound, 
            ResponseErrorType::LinkNotFoundError, 
            format!("Link with ID \"{}\" not found", link_id)
          );
        }
      }
  } else {
    response_builder.error(
      Status::InternalServerError, 
      ResponseErrorType::DatabaseError, 
      String::from("Could not get databse pool!")
    );
  }

  response_builder.build().json_respond()
}

#[patch("/edit-link", data = "<link>")]
pub fn edit_link(link: Json<models::db_less::EditLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> ResponseResult<Json<JsonErrorResponse<models::db_less::EditLinkResult>>> {
  //Err(Custom(Status::ServiceUnavailable, "Not implemented"))
  let mut response_builder: ResponseBuilder<models::db_less::EditLinkResult, Value> = ResponseBuilder::new();

  if let Ok(mut pool) = db.get() {
    let conn = &mut *pool;

    let link_id = link.link_id.clone();
    let control_key = link.control_key.clone();
    let target = link.target.clone();
    let new_link_id = link.new_link_id.clone();

    let base_url = config.base_url.clone();
    let max_id_length = config.max_id_length.clone();

    match (new_link_id.clone(), target.clone()) {
      (None, None) => {
        response_builder.error(
          Status::BadRequest,
          ResponseErrorType::ValidationError,
          String::from("No editable properties found in request data!")
        );
      },
      _ => {
        use crate::schema::links;

        let new_link_id: Result<String, ()> = if let Some(new_id) = new_link_id.clone() {
          match links::table
            .count()
            .filter(links::link_id.eq(new_id.clone()))
            .get_result::<i64>(conn) {
              Ok(count) if count == 0 => {
                if new_id.len() > max_id_length {
                  response_builder.error(
                    Status::BadRequest,
                    ResponseErrorType::ValidationError,
                    format!("ID '{}' is too long! (received length: {}, max length: {})",
                      new_id.clone(),
                      new_id.len(),
                      max_id_length
                    )
                  );
                  Err(())
                } else {
                  Ok(new_id)
                }
              },
              Ok(_) => {
                response_builder.error(
                  Status::Conflict,
                  ResponseErrorType::DuplicateIdError,
                  format!("ID '{}' is already in use!", new_id)
                );
                Err(())
              },
              Err(_) => {
                response_builder.error(
                  Status::InternalServerError,
                  ResponseErrorType::DatabaseError,
                  String::from("Could not verify uniqueness of new ID!")
                );
                Err(())
              }
            }
        } else {
          Ok(link_id.clone())
        };

        let target: Result<String, ()> = if let Some(target_str) = target.clone() {
          match Url::parse(&target_str) {
            Ok(_) => Ok(target_str),
            Err(_) => { 
              response_builder.error(
                Status::BadRequest,
                ResponseErrorType::ValidationError,
                format!("'{}' is not a valid URL!", target_str.clone())
              );
              Err(())
            }
          }
        } else {
          match links::table
            .select(links::target)
            .filter(links::link_id.eq(link_id.clone()))
            .load::<String>(conn) {
              Ok(value) if value.len() > 0 => Ok(value[0].clone()),
              Ok(_) => {
                response_builder.error(
                  Status::InternalServerError,
                  ResponseErrorType::LinkNotFoundError,
                  format!("Link with ID '{}' not found!", link_id.clone())
                );
                Err(())
              },
              Err(_) => { 
                response_builder.error(
                  Status::InternalServerError,
                  ResponseErrorType::DatabaseError,
                  String::from("Could not fetch necessary data from database!")
                );
                Err(())
              }
            }
        };

        match new_link_id {
          Ok(new_id) => match target {
            Ok(target_str) => match links::table
              .filter(links::link_id.eq(link_id.clone()))
              .load::<models::Link>(conn) {
                Ok(link) if link.len() > 0 => {
                  let link = link[0].clone();

                  match bcrypt::verify(control_key.clone(), &link.control_key) {
                    Ok(bcrypt_res) if bcrypt_res == true => {
                      match diesel::update(links::table)
                        .set((
                          links::link_id.eq(new_id.clone()),
                          links::target.eq(target_str.clone())
                        ))
                        .filter(links::link_id.eq(link_id.clone()))
                        .execute(conn) {
                          Ok(_) => { 
                            response_builder.success(Status::Ok)
                              .data(ResponseDataType::Value(models::db_less::EditLinkResult {
                                link_id: new_id.clone(),
                                target: target_str,
                                link: format!("{}/{}", base_url, new_id)
                              }));
                          },
                          Err(_) => { 
                            response_builder.error(
                              Status::InternalServerError,
                              ResponseErrorType::DatabaseError,
                              format!("Could not edit link with ID '{}' due to database error!", link_id)
                            );
                          }
                        }
                    },
                    Ok(_) => {
                      response_builder.error(
                        Status::Unauthorized,
                        ResponseErrorType::InvalidControlKeyError,
                        format!("'{}' is not a valid control key for link with ID '{}'!", control_key, link_id)
                      );
                    },
                    Err(_) => { 
                      response_builder.error(
                        Status::InternalServerError,
                        ResponseErrorType::ControlKeyHashVerificationError,
                        String::from("Could not verify validity of control key!")
                      );
                    }
                  }
                },
                Ok(_) => {
                  response_builder.error(
                    Status::InternalServerError,
                    ResponseErrorType::DatabaseError,
                    format!("Link with ID '{}' not found!", link_id)
                  );
                },
                Err(_) => { 
                  response_builder.error(
                    Status::InternalServerError,
                    ResponseErrorType::DatabaseError,
                    format!("Could not verify the presence of link with ID '{}' in database!", link_id)
                  );
                }
              },
            Err(_) => {}
          },
          Err(_) => {}
        }
      }
    }
  } else {
    response_builder.error(
      Status::InternalServerError,
      ResponseErrorType::DatabaseError,
      String::from("Could not get database pool!")
    );
  }

  response_builder.build().json_respond()
}
