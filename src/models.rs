use diesel::{Insertable, Queryable, AsChangeset};
use chrono::NaiveDateTime;
use ::serde::{Serialize};
use crate::schema::*;

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

pub mod db_less {
  use ::serde::{Serialize, Deserialize};

  #[derive(Serialize, Deserialize)]
  pub struct GetLink {
    #[serde(rename = "linkId")]
    pub link_id: String,
    pub target: String,
    #[serde(rename = "addedAt")]
    pub added_at: chrono::NaiveDateTime,
    #[serde(rename = "visitCount")]
    pub visit_count: i32
  }

  #[derive(Serialize, Deserialize)]
  pub struct NewLink {
    #[serde(rename = "linkId")]
    pub link_id: Option<String>,
    pub target: String
  }

  #[derive(Serialize, Deserialize)]
  pub struct NewLinkResult {
    #[serde(rename = "linkId")]
    pub link_id: String,
    pub target: String,
    #[serde(rename = "controlKey")]
    pub control_key: String
  }

  #[derive(Serialize, Deserialize)]
  pub struct EditLink {
    #[serde(rename = "linkId")]
    pub link_id: String,
    #[serde(rename = "newLinkId")]
    pub new_link_id: Option<String>,
    pub target: Option<String>,
    #[serde(rename = "controlKey")]
    pub control_key: String
  }

  #[derive(Serialize, Deserialize)]
  pub struct DeleteLink {
    #[serde(rename = "linkId")]
    pub link_id: String,
    #[serde(rename = "controlKey")]
    pub control_key: String
  }
}