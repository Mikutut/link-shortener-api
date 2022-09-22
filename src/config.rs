use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub database_url: String,
  pub max_requests: i64,
  pub max_requests_time_window: i64,
  pub base_url: String
}

impl Default for Config {
  fn default() -> Self {
    Config {
      database_url: "mysql://root:root@localhost:3306/link_shortener".into(),
      max_requests: 100,
      max_requests_time_window: 3600,
      base_url: String::from("http://localhost")
    }
  }
}