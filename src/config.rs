use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub database_url: String
}

impl Default for Config {
  fn default() -> Self {
    Config {
      database_url: "mysql://root:root@localhost:3306/link_shortener".into()
    }
  }
}