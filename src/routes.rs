pub mod root {
  use crate::models;
  use rocket::response::{Redirect, status::Custom};
  use rocket::{self, get, post, delete, patch, State};
  use rocket::http::Status;
  use rocket::serde::{json::Json};
  use nanoid::{nanoid};
  use crate::fairings::database::Pool;
  use crate::guards;

  use diesel::QueryDsl;
  use diesel::prelude::*;

  #[get("/<link_id>")]
  pub fn access_link(link_id: String, db: &State<Pool>, rl: guards::rate_limit::RateLimit) -> Result<Redirect, &'static str> {
    if let Ok(mut pool) = db.get() {
      let conn = &mut *pool;

      use crate::schema::links;

      let result = links::table
        .find(link_id)
        .limit(1)
        .load::<models::Link>(conn);

      if let Ok(link) = result {
        let target = link[0].clone().target;

        Ok(Redirect::to(target))
      } else {
        Err("Link not found!")
      }
    } else {
      Err("Could not get database pool!")
    }
  }

  #[get("/get-links")]
  pub fn get_links(db: &State<Pool>, rl: guards::rate_limit::RateLimit) -> Json<Vec<models::db_less::GetLink>> {
    let mut pool = db.get().expect("Could not get database pool!");
    let conn = &mut *pool;

    use crate::schema::links;

    let results = links::table
      .order(links::link_id.asc())
      .load::<models::Link>(conn)
      .expect("Could not load links!");

    Json(results.iter().map(|r| {
      models::db_less::GetLink {
        link_id: r.link_id.clone(),
        target: r.target.clone(),
        added_at: r.added_at,
        visit_count: r.visit_count
      }
    }).collect::<Vec<models::db_less::GetLink>>())
  }

  #[post("/add-link", data = "<link>", rank = 1)]
  pub fn add_link(link: Json<models::db_less::NewLink>, db: &State<Pool>, rl: guards::rate_limit::RateLimit) -> Result<Json<models::Link>, &'static str> {
    let mut pool = db.get().expect("Could not get database pool!");
    let conn = &mut *pool;

    use crate::schema::links;

    let mut new_link_id = nanoid!(12);
    let new_control_key = nanoid!(24);
    let new_target = link.0.target;

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
        control_key: new_control_key
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

          Ok(Json(new_link))
        } else {
          Err("New link was added, but could not retrieve required data.")
        }
      } else {
        Err("Could not add link to database!")
      }
    } else {
      return Err("Could not send database query!");
    }
  }

  #[delete("/delete-link", data = "<link>")]
  pub fn delete_link(link: Json<models::db_less::DeleteLink>, db: &State<Pool>, rl: guards::rate_limit::RateLimit) -> Result<(), &'static str> {
    if let Ok(mut pool) = db.get() {
      let conn = &mut *pool;
      use crate::schema::links;

      let link_id = link.0.link_id;
      let control_key = link.0.control_key;

      let result = links::table
        .filter(links::link_id.eq(link_id.clone()))
        .count()
        .get_result::<i64>(conn);

      if let Ok(link_count) = result {

        if link_count == 1 {
          let result = links::table
            .filter(links::link_id.eq(link_id.clone()).and(links::control_key.eq(control_key)))
            .count()
            .get_result::<i64>(conn);

          if let Ok(count) = result {
            if count == 1 {
              let result = diesel::delete(links::table
                .filter(links::link_id.eq(link_id.clone()))
              )
                .execute(conn);

              if let Ok(_) = result {
                Ok(())
              } else {
                Err("Could not delete link!")
              }
            } else {
              Err("Invalid control key!")
            }
          } else {
            Err("Could not verify control key!")
          }
        } else {
          Err("Link not found")
        }
      } else {
        Err("Link not found")
      }
    } else {
      Err("Could not get database pool!")
    }
  }

  #[patch("/edit-link", data = "<link>")]
  pub fn edit_link(link: Json<models::db_less::EditLink>, db: &State<Pool>, rl: guards::rate_limit::RateLimit) -> Result<Json<models::Link>, Custom<&'static str>> {
    //Err(Custom(Status::ServiceUnavailable, "Not implemented"))
    if let Ok(mut pool) = db.get() {
      use crate::schema::links;
      let conn = &mut *pool;

      let link_id = link.link_id.clone();
      let control_key = link.control_key.clone();
      let target = link.target.clone();
      let new_link_id = link.new_link_id.clone();

      if target.is_none() && new_link_id.is_none() {
        Err(Custom(Status::UnprocessableEntity, "No editable properties found in request data!"))
      } else {
        match links::table
          .filter(links::link_id.eq(link_id.clone()))
          .count()
          .get_result::<i64>(conn) {
            Ok(count) => {
              if count == 1 {
                match links::table
                  .filter(links::link_id.eq(link_id.clone()).and(links::control_key.eq(control_key.clone())))
                  .count()
                  .get_result::<i64>(conn) {
                    Ok(count) => {
                      if count == 1 {
                        if let Some(new_link_id) = new_link_id {
                          match links::table
                            .filter(links::link_id.eq(new_link_id.clone()))
                            .count()
                            .get_result::<i64>(conn) {
                              Ok(count) => {
                                if count == 0 {
                                  if let Some(target) = target {
                                    match diesel::update(links::table)
                                      .filter(links::link_id.eq(link_id.clone()))
                                      .set((links::link_id.eq(new_link_id.clone()), links::target.eq(target)))
                                      .execute(conn) {
                                        Ok(_) => {
                                          match links::table
                                            .filter(links::link_id.eq(new_link_id))
                                            .load::<models::Link>(conn) {
                                              Ok(link_res) => {
                                                let link_res = link_res[0].clone();

                                                Ok(Json(link_res))
                                              },
                                              Err(_) => {
                                                Err(Custom(Status::InternalServerError, "Server has applied changes to the link but it could not retrieve result! Contact the administrator!"))
                                              }
                                            }
                                        },
                                        Err(_) => {
                                          Err(Custom(Status::InternalServerError, "Could not apply changes to the link! Please try again later!"))
                                        }
                                      }
                                  } else {
                                    match diesel::update(links::table)
                                      .filter(links::link_id.eq(link_id.clone()))
                                      .set(links::link_id.eq(new_link_id.clone()))
                                      .execute(conn) {
                                        Ok(_) => {
                                          match links::table
                                            .filter(links::link_id.eq(new_link_id))
                                            .load::<models::Link>(conn) {
                                              Ok(link_res) => {
                                                let link_res = link_res[0].clone();

                                                Ok(Json(link_res))
                                              },
                                              Err(_) => {
                                                Err(Custom(Status::InternalServerError, "Server has applied changes to the link but it could not retrieve result! Contact the administrator!"))
                                              }
                                            }
                                        },
                                        Err(_) => {
                                          Err(Custom(Status::InternalServerError, "Could not apply changes to the link! Please try again later!"))
                                        }
                                      }
                                  }
                                } else {
                                  Err(Custom(Status::Conflict, "Link with provided ID already exists. Please choose another one."))
                                }
                              },
                              Err(_) => {
                                Err(Custom(Status::InternalServerError, "Could not verify new link ID's uniqueness!"))
                              }
                            }
                        } else if let Some(target) = target {
                          match diesel::update(links::table)
                            .filter(links::link_id.eq(link_id.clone()))
                            .set(links::target.eq(target))
                            .execute(conn) {
                              Ok(_) => {
                                match links::table
                                  .filter(links::link_id.eq(link_id))
                                  .load::<models::Link>(conn) {
                                    Ok(link_res) => {
                                      let link_res = link_res[0].clone();

                                      Ok(Json(link_res))
                                    },
                                    Err(_) => {
                                      Err(Custom(Status::InternalServerError, "Server has applied changes to the link but it could not retrieve result! Contact the administrator!"))
                                    }
                                  }
                              },
                              Err(_) => {
                                Err(Custom(Status::InternalServerError, "Could not apply changes to the link! Please try again later!"))
                              }
                            }
                        } else {
                          Err(Custom(Status::InternalServerError, "Rust is a bitch (an error, which should not have happened, happened)"))
                        }
                      } else {
                        Err(Custom(Status::Unauthorized, "Invalid control key!"))
                      }
                    },
                    Err(_) => {
                      Err(Custom(Status::InternalServerError, "Could not verify control key!"))
                    }
                  }
              } else {
                Err(Custom(Status::NotFound, "Link ID not found!"))
              }
            },
            Err(_) => {
              Err(Custom(Status::InternalServerError, "Could not verify link's presence!"))
            }
          }
      }

    } else {
      Err(Custom(Status::InternalServerError, "Could not get database pool!"))
    }
  }
}