use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel::mysql::MysqlConnection;
use crate::config;
use rocket::{Rocket, Build, fairing::{self, Fairing}};

pub type Pool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

pub struct DatabaseInitiator;

#[rocket::async_trait]
impl Fairing for DatabaseInitiator {
  fn info(&self) -> fairing::Info {
    fairing::Info {
      name: "Database Pool Initiator",
      kind: fairing::Kind::Ignite
    }
  }

  async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
    match rocket.state::<config::Config>() {
      Some(config) => {
        let database_url = config.database_url.clone();

        let manager = ConnectionManager::<MysqlConnection>::new(&database_url);
        match r2d2::Pool::builder().build(manager) {
          Ok(pool) => {
            println!("Database pool initialized!");

            fairing::Result::Ok(rocket.manage(pool))
          },
          Err(_) => {
            println!("Could not initialize database pool!");
            fairing::Result::Err(rocket)
          }
        }
      },
      None => {
        fairing::Result::Err(rocket)
      }
    }
  }
}