use figment::providers::{Format, Toml};
use rocket::{self, launch, routes, fairing::AdHoc, catchers};
use link_shortener_api::fairings;
use link_shortener_api::config::Config;
use link_shortener_api::catchers;
use link_shortener_api::routes;

#[launch]
fn rocket() -> _ {
  let figment = rocket::config::Config::figment()
    .join(Toml::file("Config.toml").nested());

  rocket::custom(figment)
    .attach(AdHoc::config::<Config>())
    .attach(fairings::database::DatabaseInitiator)
    .attach(fairings::rate_limit::RateLimit)
    .mount("/", routes![
    //  routes::get_get_links, 
    //  routes::post_add_link,
    //  routes::put_add_link, 
      routes::root::get_access_link, 
    //  routes::delete_delete_link,
    //  routes::patch_edit_link,
    //  routes::post_edit_link,
      routes::root::get_check_id
    ])
    .register("/", catchers![
      catchers::root::invalid_request_data,
      catchers::root::rate_limited,
      catchers::root::default_catcher
    ])
    //.mount("/bulk", routes![
    //  routes::bulk::put_add_link,
    //  routes::bulk::post_add_link
    //])
}