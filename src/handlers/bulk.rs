use rocket::{http::Status, State};
use diesel::prelude::*;
use bcrypt;
use crate::requests;
use crate::responses::*;
use crate::config::Config;
use crate::fairings::database::Pool;
use crate::models;

pub fn add_links(links: Vec<requests::NewLink>, db: &State<Pool>, config: &State<Config>) -> Response<Vec<successes::NewLinkResult>> {
  let mut res_data = ResponseData::<Vec<successes::NewLinkResult>>::new();
  let mut new_links: Vec<successes::NewLinkResult> = Vec::new();
  let mut success = true;
  let mut i: u32 = 0;

  for link in links.iter() {
    i += 1;
    match super::utils::add_link(link, db, config) {
      Ok(r) => {
        new_links.push(r);
      },
      Err(e) => {
        res_data = errors::Errors::bulk_request_error(
          res_data, 
          e.clone_status(), 
          i, 
          e
        );
        success = false;
        break;
      }
    }
  }

  if success {
    match db.get() {
      Ok(mut pool) => {
        use crate::schema::links;
        let conn = &mut *pool;
        let mut success = true;
        let mut i: u32 = 0;
        let mut new_links_db: Vec<models::NewLink> = Vec::new();

        for new_link in new_links.iter() {
          i += 1;
          let link_id = &new_link.link_id;
          let control_key = &new_link.control_key;
          let target = &new_link.target;

          match bcrypt::hash(control_key, bcrypt::DEFAULT_COST) {
            Ok(hash) => {
              let new_link_db = models::NewLink {
                link_id: link_id.clone(),
                control_key: hash,
                target: target.clone()
              };

              new_links_db.push(new_link_db);
            },
            Err(_) => {
              let req_data: ResponseData<Vec<successes::NewLinkResult>> = ResponseData::new().error(
                Status::InternalServerError,
                ResponseErrorType::ControlKeyHashGenerationError,
                format!("Could not generate bcrypt hash of control key for link with ID '{}'.", link_id),
                None
              );

              res_data = errors::Errors::bulk_request_error(
                res_data, 
                req_data.clone_status(), 
                i, 
                req_data
              );
              success = false;
              break;
            }
          }
        }

        if success {
          match diesel::insert_into(links::table)
            .values(&new_links_db)
            .execute(conn) {
              Ok(_) => {
                res_data = res_data.success(
                  Status::Ok,
                  Some(new_links)
                );
              },
              Err(_) => {
                res_data = res_data.error(
                  Status::InternalServerError,
                  ResponseErrorType::DatabaseError,
                  String::from("Could not add links to database!"),
                  None
                );
              }
            }
        }
      },
      Err(_) => {
        res_data = errors::Errors::database_pool(res_data);
      }
    }
  }

  res_data
    .to_response()
}