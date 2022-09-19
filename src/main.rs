use figment::providers::{Format, Toml};
use rocket::{self, launch, routes, fairing::AdHoc, catchers};
use link_shortener_api::{config::{Config}, catchers, routes, fairings};

#[launch]
fn rocket() -> _ {
  let figment = rocket::config::Config::figment()
    .join(Toml::file("Config.toml").nested());

  rocket::custom(figment)
    .attach(AdHoc::config::<Config>())
    .attach(fairings::database::DatabaseInitiator)
    .attach(fairings::rate_limit::RateLimit)
    .mount("/", routes![
      routes::root::get_links, 
      routes::root::add_link, 
      routes::root::access_link, 
      routes::root::delete_link,
      routes::root::edit_link
    ])
    .register("/", catchers![
      catchers::root::invalid_request_data
    ])
}
