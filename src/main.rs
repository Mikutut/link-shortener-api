use figment::providers::{Format, Toml};
use rocket::{self, launch, routes, fairing::AdHoc, catchers};
use link_shortener_api::{config::{Config}, catchers, routes, fairings};

use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel::mysql::MysqlConnection;

#[launch]
fn rocket() -> _ {
  let figment = rocket::config::Config::figment()
    .join(Toml::file("Config.toml").nested());

  rocket::custom(figment)
    .attach(AdHoc::config::<Config>())
    .attach(AdHoc::on_ignite("Database Pool Initiator", |rocket| Box::pin(async {
      let config = rocket.state::<Config>().expect("Could not get config information!");
      let database_url = config.database_url.clone();

      let manager = ConnectionManager::<MysqlConnection>::new(&database_url);
      let pool: routes::Pool = r2d2::Pool::builder().build(manager).expect("Could not create connection pool!");

      rocket.manage(pool)
    })))
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
