//TODO: Replace all occurrences of old response system
use crate::models;
use rocket::response::{Redirect, self};
use rocket::{self, get, post, delete, patch, State};
use rocket::http::Status;
use rocket::serde::{json::Json};
use nanoid::{nanoid};
use crate::fairings::database::Pool;
use crate::{guards, responses::{self, Result as ResponseResult}};
use bcrypt;
use url::{Url};

use diesel::QueryDsl;
use diesel::prelude::*;

//TODO: Add new responses
#[get("/<link_id>")]
pub fn access_link(link_id: String, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> Result<Redirect, responses::Error> {
  if let Ok(mut pool) = db.get() {
    let conn = &mut *pool;

    use crate::schema::links;

    let result = links::table
      .find(link_id.clone())
      .limit(1)
      .load::<models::Link>(conn);

    if let Ok(link) = result {
      let target = link[0].clone().target;

      Ok(Redirect::temporary(target))
    } else {
      Err(
        (Status::NotFound, Json(responses::ResponseError {
          error_type: responses::ResponseErrorType::LinkNotFoundError,
          error_message: format!("Link with ID \"{}\" not found!", link_id)
        }))
      )
    }
  } else {
    Err((
      Status::InternalServerError, 
      Json(responses::ResponseError {
        error_type: responses::ResponseErrorType::DatabaseError,
        error_message: String::from("Could not get database pool!")
      })
    ))
  }
}

//TODO: Add new responses
#[get("/get-links")]
pub fn get_links(db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> responses::new::ResponseResult<Json<responses::new::JsonErrorResponse<Vec<models::db_less::GetLink>>>> {
  use responses::new::*;
  let mut response_builder = responses::new::ResponseBuilder::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;

      let conn = &mut *pool;

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
                    visit_count: r.visit_count.clone()
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

//TODO: Handler generates new link ID even if one was provided with request
//TODO: Add new responses
#[post("/add-link", data = "<link>", rank = 1)]
pub fn add_link(link: Json<models::db_less::NewLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> responses::new::ResponseResult<Json<responses::new::JsonErrorResponse<models::db_less::NewLinkResult>>> {
  use responses::new::*;

  let mut response_builder = ResponseBuilder::new();

  match db.get() {
    Ok(mut pool) => {
      use crate::schema::links;
      let conn = &mut *pool;

      let mut new_link_id = nanoid!(12);
      let new_control_key = nanoid!(24);
      let new_target = link.0.target;

      match bcrypt::hash(new_control_key.clone(), bcrypt::DEFAULT_COST) {
        Ok(new_control_key_hash) => {
          let result = links::table
            .select(links::link_id)
            .load::<String>(conn);

          if let Ok(results) = result {
            while results.contains(&new_link_id) {
              new_link_id = nanoid!(12);
            }

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
                    link_id: new_link.link_id,
                    target: new_link.target,
                    control_key: new_control_key
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

//TODO: Add new responses
#[delete("/delete-link", data = "<link>")]
pub fn delete_link(link: Json<models::db_less::DeleteLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> Result<(), responses::Error> {
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
                    Ok(())
                  },
                  Err(_) => {
                    Err((
                      Status::InternalServerError,
                      responses::ResponseError::new(
                        responses::ResponseErrorType::DatabaseError,
                        format!("Could not delete \"{}\" link from database!", link_id)
                      ).to_json()
                    ))
                  }
                }
            },
            _ => {
              Err((
                Status::Unauthorized,
                responses::ResponseError::new(
                  responses::ResponseErrorType::InvalidControlKeyError,
                  format!("\"{}\" is not a valid control key for \"{}\" link!", control_key, link_id)
                ).to_json()
              ))
            }
          }
        },
        _ => {
          Err((
            Status::NotFound,
            responses::ResponseError::new(
              responses::ResponseErrorType::LinkNotFoundError,
              format!("Link with ID \"{}\" not found", link_id)
            ).to_json()
          ))
        }
      }
  } else {
    Err((
      Status::InternalServerError, 
      Json(responses::ResponseError {
        error_type: responses::ResponseErrorType::DatabaseError,
        error_message: String::from("Could not get database pool!")
      })
    ))
  }
}

// TODO: Add new responses
#[patch("/edit-link", data = "<link>")]
pub fn edit_link(link: Json<models::db_less::EditLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> ResponseResult<models::db_less::EditLinkResult> {
  //Err(Custom(Status::ServiceUnavailable, "Not implemented"))
  if let Ok(mut pool) = db.get() {
    let conn = &mut *pool;

    let link_id = link.link_id.clone();
    let control_key = link.control_key.clone();
    let target = link.target.clone();
    let new_link_id = link.new_link_id.clone();

    match (new_link_id.clone(), target.clone()) {
      (None, None) => {
        Err((
          Status::BadRequest,
          responses::ResponseError::new(
            responses::ResponseErrorType::ValidationError,
            String::from("No editable properties found in request data!")
          ).to_json()
        ))
      },
      _ => {
        use crate::schema::links;

        let new_link_id = if let Some(new_id) = new_link_id.clone() {
          match links::table
            .count()
            .filter(links::link_id.eq(new_id.clone()))
            .get_result::<i64>(conn) {
              Ok(count) if count == 0 => {
                if new_id.len() > 255 {
                  Err((
                    Status::BadRequest,
                    responses::ResponseError::new(
                      responses::ResponseErrorType::ValidationError,
                      format!("ID '{}' is too long! (received length: {}, max length: {})",
                        new_id.clone(),
                        new_id.len(),
                        255
                      )
                    ).to_json()
                  ))
                } else {
                  Ok(new_id)
                }
              },
              Ok(_) => {
                Err((
                  Status::Conflict,
                  responses::ResponseError::new(
                    responses::ResponseErrorType::DuplicateIdError,
                    format!("ID '{}' is already in use!", new_id)
                  ).to_json()
                ))
              },
              Err(_) => {
                Err((
                  Status::InternalServerError,
                  responses::ResponseError::new(
                    responses::ResponseErrorType::DatabaseError,
                    String::from("Could not verify uniqueness of new ID!")
                  ).to_json()
                ))
              }
            }
        } else {
          Ok(link_id.clone())
        };

        let target = if let Some(target_str) = target.clone() {
          match Url::parse(&target_str) {
            Ok(_) => Ok(target_str),
            Err(_) => Err((
              Status::BadRequest,
              responses::ResponseError::new(
                responses::ResponseErrorType::ValidationError,
                format!("'{}' is not a valid URL!", target_str.clone())
              ).to_json()
            ))
          }
        } else {
          match links::table
            .select(links::target)
            .filter(links::link_id.eq(link_id.clone()))
            .load::<String>(conn) {
              Ok(value) if value.len() > 0 => Ok(value[0].clone()),
              Ok(_) => Err((
                Status::NotFound,
                responses::ResponseError::new(
                  responses::ResponseErrorType::LinkNotFoundError,
                  format!("Link with ID '{}' not found!", link_id.clone())
                ).to_json()
              )),
              Err(_) => Err((
                Status::InternalServerError,
                responses::ResponseError::new(
                  responses::ResponseErrorType::DatabaseError,
                  String::from("Could not fetch necessary data from database!")
                ).to_json()
              ))
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
                          Ok(_) => Ok((
                            Status::Ok,
                            Json(models::db_less::EditLinkResult {
                              link_id: new_id,
                              target: target_str
                            })
                          )),
                          Err(_) => Err((
                            Status::InternalServerError,
                            responses::ResponseError::new(
                              responses::ResponseErrorType::DatabaseError,
                              format!("Could not edit link with ID '{}' due to database error!", link_id)
                            ).to_json()
                          ))
                        }
                    },
                    Ok(_) => Err((
                      Status::Unauthorized,
                      responses::ResponseError::new(
                        responses::ResponseErrorType::InvalidControlKeyError,
                        format!("'{}' is not a valid control key for link with ID '{}'!",
                          control_key,
                          link_id
                        )
                      ).to_json()
                    )),
                    Err(_) => Err((
                      Status::InternalServerError,
                      responses::ResponseError::new(
                        responses::ResponseErrorType::ControlKeyHashVerificationError,
                        String::from("Could not verify validity of control key!")
                      ).to_json()
                    ))
                  }
                },
                Ok(_) => Err((
                  Status::NotFound,
                  responses::ResponseError::new(
                    responses::ResponseErrorType::LinkNotFoundError,
                    format!("Link with ID '{}' not found!", link_id)
                  ).to_json()
                )),
                Err(_) => Err((
                  Status::InternalServerError,
                  responses::ResponseError::new(
                    responses::ResponseErrorType::DatabaseError,
                    format!("Could not verify the presence of link with ID '{}' in database!", link_id)
                  ).to_json()
                ))
              },
            Err(err) => Err(err)
          },
          Err(err) => Err(err)
        }
      }
    }
  } else {
    Err((
      Status::InternalServerError,
      Json(responses::ResponseError::new(responses::ResponseErrorType::DatabaseError, "Could not get database pool!".into()))
    ))
  }
}
