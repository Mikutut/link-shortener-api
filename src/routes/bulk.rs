use rocket::{post, put, State};
use rocket::serde::json::Json;
use rocket::http::Status;
use crate::guards::rate_limit::RateLimit;
use crate::fairings::database::Pool;
use crate::responses::*;
use crate::requests;
use crate::config::Config;
use crate::handlers;

#[put("/add-link", data = "<links>")]
pub fn put_add_link(links: Json<Vec<requests::NewLink>>, db: &State<Pool>, _rl: RateLimit, config: &State<Config>) -> (Status, Json<Response<Vec<successes::NewLinkResult>>>) {
  let links = links.into_inner();

  handlers::bulk::add_links(links, db, config)
    .json_respond()
}
#[post("/add-link", data = "<links>")]
pub fn post_add_link(links: Json<Vec<requests::NewLink>>, db: &State<Pool>, _rl: RateLimit, config: &State<Config>) -> (Status, Json<Response<Vec<successes::NewLinkResult>>>) {
  let links = links.into_inner();

  handlers::bulk::add_links(links, db, config)
    .json_respond()
}