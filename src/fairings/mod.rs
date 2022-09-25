pub mod database {
  use diesel::r2d2;
  use diesel::r2d2::ConnectionManager;
  use diesel::mysql::MysqlConnection;
  use crate::config;
  use rocket::{Rocket, Build, fairing::{self, Fairing}};

  pub type Pool = r2d2::Pool<ConnectionManager<MysqlConnection>>;
  pub type PoolConnection = r2d2::PooledConnection<ConnectionManager<MysqlConnection>>;

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
              println!("Could not initialze database pool!");
              fairing::Result::Err(rocket)
            }
          }          
        },
        None => fairing::Result::Err(rocket)
      }
    }
  }
}

pub mod rate_limit {
  use std::{collections::HashMap, net::IpAddr};
  use chrono::{NaiveDateTime, Utc};
  use rocket::fairing::{self, Fairing, Result};
  use rocket::{Rocket, Build, Request, Data, State};
  use rocket::outcome::Outcome;
  use std::sync::Mutex;
  use crate::config;

  pub type RateLimitState = Mutex<HashMap<IpAddr, (i64, NaiveDateTime)>>;

  pub struct RateLimit;
  #[rocket::async_trait]
  impl Fairing for RateLimit {
    fn info(&self) -> fairing::Info {
      fairing::Info {
        name: "Rate Limiter",
        kind: fairing::Kind::Ignite | fairing::Kind::Request
      }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
      let state: RateLimitState = Mutex::new(HashMap::new());

      Ok(rocket.manage(state))
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
      match req.guard::<&State<config::Config>>().await {
        Outcome::Success(config) => {
          match req.guard::<&State<RateLimitState>>().await {
            Outcome::Success(state) => {
              let time_window = config.max_requests_time_window.clone();
              let max_requests = config.max_requests.clone();

              match req.client_ip() {
                Some(ip) => {
                  match state.lock() {
                    Ok(mut lock) => {
                      match lock.contains_key(&ip) {
                        true => {
                          println!("Found \"{}\" in rate limit state.", ip.to_string());
                          match lock.get_mut(&ip) {
                            Some(entry) => {
                              println!("\"{}\"'s entry - {} requests, last request: {}", ip.to_string(), entry.0, entry.1.to_string());

                              if entry.0 < max_requests {
                                println!("\"{}\" is not exceeding max requests limit. Reseting date...", ip.to_string());

                                entry.1 = Utc::now().naive_utc();
                              } else {
                                println!("\"{}\" exceeded max requests limit.", ip.to_string());

                                let current_time = Utc::now().naive_utc();
                                let diff = current_time.signed_duration_since(entry.1).num_seconds();

                                if diff >= time_window {
                                  println!("Time window reached for \"{}\". Unlocking...", ip.to_string());
                                  entry.0 = 1;
                                } else {
                                  println!("\"{}\" still within time window. Blocking...", ip.to_string());
                                }
                              }
                            },
                            None => {
                              println!("Could not get entry from rate limit state!");
                            }
                          }
                        },
                        false => {
                          println!("Adding \"{}\" to rate limit state.", ip.to_string());
                          lock.insert(ip, (0, Utc::now().naive_utc()));
                        }
                      }
                    },
                    Err(_) => {
                      println!("Could not acquire lock on rate limit state!");
                    }
                  }
                },
                None => {
                  println!("Could not get client ip!");
                }
              }
            },
            _ => {
              println!("Could not retrieve rate limit state!");
            }
          }
        },
        _ => {
          println!("Could not acquire config!");
        }
      }
    }
  }
}