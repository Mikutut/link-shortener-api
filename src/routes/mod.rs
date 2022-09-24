use crate::models;
use rocket::response::{Redirect};
use rocket::{self, get, post, delete, patch, put, State};
use rocket::serde::{json::{Json}};
use crate::fairings::database::Pool;
use crate::{guards, responses::*, config::Config, handlers};

pub mod bulk;

#[get("/check-id/<link_id>")]
pub fn get_check_id(link_id: String, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> ResponseResult<Json<JsonErrorResponse<bool>>> {
  handlers::check_id(link_id, db)
}

#[get("/<link_id>")]
pub fn get_access_link(link_id: String, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> Result<Redirect, ResponseResult<Json<JsonErrorResponse<()>>>> {
  handlers::access_link(link_id, db)
}

#[get("/get-links")]
pub fn get_get_links(db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> ResponseResult<Json<JsonErrorResponse<Vec<models::db_less::GetLink>>>> {
  handlers::get_links(db, config)
}

#[post("/add-link", data = "<link>")]
pub fn post_add_link(link: Json<models::db_less::NewLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> ResponseResult<Json<JsonErrorResponse<models::db_less::NewLinkResult>>> {
  handlers::add_link(link, db, config)
}

#[put("/add-link", data = "<link>")]
pub fn put_add_link(link: Json<models::db_less::NewLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> ResponseResult<Json<JsonErrorResponse<models::db_less::NewLinkResult>>> {
  handlers::add_link(link, db, config)
}

#[delete("/delete-link", data = "<link>")]
pub fn delete_delete_link(link: Json<models::db_less::DeleteLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit) -> ResponseResult<Json<JsonErrorResponse<()>>> {
  handlers::delete_link(link, db)
}

#[patch("/edit-link", data = "<link>")]
pub fn patch_edit_link(link: Json<models::db_less::EditLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> ResponseResult<Json<JsonErrorResponse<models::db_less::EditLinkResult>>> {
  handlers::edit_link(link, db, config)
}

#[post("/edit-link", data = "<link>")]
pub fn post_edit_link(link: Json<models::db_less::EditLink>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>) -> ResponseResult<Json<JsonErrorResponse<models::db_less::EditLinkResult>>> {
  handlers::edit_link(link, db, config)
}
