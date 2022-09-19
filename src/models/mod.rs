use diesel::{Insertable, Queryable, AsChangeset};
use chrono::NaiveDateTime;
use ::serde::{Serialize};
use crate::schema::*;

pub mod db_less;

#[derive(Queryable, AsChangeset, Serialize, Clone)]
pub struct Link {
  #[serde(rename = "linkId")]
  pub link_id: String,
  pub target: String,
  #[serde(rename = "controlKey")]
  pub control_key: String,
  #[serde(rename = "addedAt")]
  pub added_at: NaiveDateTime,
  #[serde(rename = "visitCount")]
  pub visit_count: i32
}

#[derive(Insertable)]
#[diesel(table_name = links)]
pub struct NewLink {
  pub link_id: String,
  pub target: String,
  pub control_key: String
}

#[derive(AsChangeset)]
#[diesel(table_name = links)]
pub struct UpdateVisitCountLinks {
  pub visit_count: i32
}