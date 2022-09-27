use rocket::{self, get, post, put, patch, delete, State, response::Redirect};
use rocket::serde::{json::{Json}};
use rocket::http::Status;
use crate::fairings::database::Pool;
use crate::{guards, responses::*, handlers};
use crate::requests;
use crate::config::Config;

#[get("/check-id/<link_id>")]
pub fn get_check_id(link_id: String, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> (Status, Json<Response<bool>>) {
  let res_data = ResponseData::<bool>::new();

  match handlers::utils::check_id::<successes::NewLinkResult>(&link_id, db) {
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
      r.transform::<bool>(Some(false))
        .clear_data()
        .to_response()
        .json_respond()
    }
  }
}

#[get("/l/<link_id>")]
pub fn get_access_link(link_id: String, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> Result<Redirect, (Status, Json<Response<()>>)> {
  match handlers::root::access_link(link_id, db) {
    Ok(redirect) => Ok(redirect),
    Err(response) => Err(response.json_respond())
  }
}

#[get("/get-links")]
pub fn get_get_links(db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> (Status, Json<Response<Vec<successes::GetLink>>>) {
  let res_data = ResponseData::new();

  match handlers::utils::get_links(db, config) {
    Ok(links) => {
      res_data.success(Status::Ok, Some(links))
        .to_response()
        .json_respond()
    },
    Err(r) => {
      r
        .transform(None)
        .to_response()
        .json_respond()
    }
  }
}

#[post("/add-link", data = "<link>")]
pub fn post_add_link(link: Json<requests::NewLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> (Status, Json<Response<successes::NewLinkResult>>) {
  let link = link.into_inner();

  handlers::root::add_link(&link, db, config)
}
#[put("/add-link", data = "<link>")]
pub fn put_add_link(link: Json<requests::NewLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> (Status, Json<Response<successes::NewLinkResult>>) {
  let link = link.into_inner();

  handlers::root::add_link(&link, db, config)
}

#[delete("/delete-link", data = "<link>")]
pub fn delete_delete_link(link: Json<requests::DeleteLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> (Status, Json<Response<()>>) {
  let res_data = ResponseData::new();
  let link_id = link.link_id.clone();
  let control_key = link.control_key.clone();

  match handlers::root::delete_link(&link_id, &control_key, db) {
    Ok(()) => {
      res_data.success(Status::Ok, None)
        .to_response()
        .json_respond()
    },
    Err(r) => r.to_response().json_respond()
  }
}

#[post("/edit-link", data = "<link>")]
pub fn post_edit_link(link: Json<requests::EditLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> (Status, Json<Response<successes::EditLinkResult>>) {
  let res_data = ResponseData::new();
  let link_id = link.link_id.clone();
  let control_key = link.control_key.clone();
  let new_link_id = link.new_link_id.clone();
  let new_target = link.target.clone();
 
  match handlers::root::edit_link(&link_id, &control_key, &new_link_id, &new_target, db, config) {
    Ok(r) => {
      res_data.success(
        Status::Ok,
        Some(r)
      )
        .to_response()
        .json_respond()
    },
    Err(r) => {
      r.transform::<successes::EditLinkResult>(None)
        .to_response()
        .json_respond()
    }
  }
}
#[patch("/edit-link", data = "<link>")]
pub fn patch_edit_link(link: Json<requests::EditLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> (Status, Json<Response<successes::EditLinkResult>>) {
  let res_data = ResponseData::new();
  let link_id = link.link_id.clone();
  let control_key = link.control_key.clone();
  let new_link_id = link.new_link_id.clone();
  let new_target = link.target.clone();
 
  match handlers::root::edit_link(&link_id, &control_key, &new_link_id, &new_target, db, config) {
    Ok(r) => {
      res_data.success(
        Status::Ok,
        Some(r)
      )
        .to_response()
        .json_respond()
    },
    Err(r) => {
      r.transform::<successes::EditLinkResult>(None)
        .to_response()
        .json_respond()
    }
  }
}