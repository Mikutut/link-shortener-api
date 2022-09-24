use rocket::{post, put, State};
use rocket::serde::json::{Json, Value};
use crate::{models, handlers};
use crate::fairings::database::Pool;
use crate::fairings::rate_limit::RateLimitState;
use crate::guards;
use crate::config::Config;
use crate::responses::*;
use std::net::SocketAddr;

#[put("/add-link", data = "<links>")]
pub fn put_add_link(links: Json<Vec<models::db_less::NewLink>>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>, rl: &State<RateLimitState>, ip: SocketAddr) -> ResponseResult<Json<Response<Vec<models::db_less::NewLinkResult>, Value>>> {
  let response_builder = handlers::bulk::add_link(links, db, config, rl, ip);

  response_builder.build().json_respond()
}

#[post("/add-link", data = "<links>")]
pub fn post_add_link(links: Json<Vec<models::db_less::NewLink>>, db: &State<Pool>, _rl: guards::rate_limit::RateLimit, config: &State<Config>, rl: &State<RateLimitState>, ip: SocketAddr) -> ResponseResult<Json<Response<Vec<models::db_less::NewLinkResult>, Value>>> {
  let response_builder = handlers::bulk::add_link(links, db, config, rl, ip);

  response_builder.build().json_respond()
}