pub mod rate_limit {
  use std::{collections::HashMap, net::IpAddr};
  use chrono::NaiveDateTime;

  pub type RateLimitState = HashMap<IpAddr, (i64, NaiveDateTime)>;
}