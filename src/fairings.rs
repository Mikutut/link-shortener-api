pub mod rate_limit {
  use std::{collections::HashMap, net::IpAddr};
  use chrono::{NaiveDateTime, Utc};
  use rocket::{fairing::{self, Fairing, Result}, Rocket, Build, Request, Data, State, outcome::Outcome};
  use std::sync::Mutex;
  use crate::config;

  pub type RateLimitState = HashMap<IpAddr, (i64, NaiveDateTime)>;

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
      let state: Mutex<RateLimitState> = Mutex::new(HashMap::new());

      Ok(rocket.manage(state))
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
      match req.guard::<&State<config::Config>>().await {
        Outcome::Success(config) => {
          match req.guard::<&State<Mutex<RateLimitState>>>().await {
            Outcome::Success(state) => {
              let time_window = config.max_requests_time_window.clone();
              let max_requests = config.max_requests.clone();

              match req.client_ip() {
                Some(ip) => {
                  let mut lock = state.lock().expect("lock rate limit state");

                  if !lock.contains_key(&ip) {
                    println!("Adding \"{}\" to rate limit state.", ip.to_string());
                    lock.insert(ip, (0, Utc::now().naive_utc()));
                  } else {
                    println!("\"{}\" already in rate limit state.", ip.to_string());

                    match lock.get_mut(&ip) {
                      Some(entry) => {
                        println!("Data for \"{}\": {} requests | Last request date: {}", ip.to_string(), entry.0, entry.1.to_string());

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
                        println!("Could not acquire entry!");
                      }
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
          ()
        }
      }
    } 
  }
}