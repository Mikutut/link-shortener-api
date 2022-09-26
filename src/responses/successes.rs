use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GetLink {
  #[serde(rename = "linkId")]
  pub link_id: String,
  pub target: String,
  #[serde(rename = "addedAt")]
  pub added_at: chrono::NaiveDateTime,
  #[serde(rename = "visitCount")]
  pub visit_count: i32,
  pub link: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewLinkResult {
  #[serde(rename = "linkId")]
  pub link_id: String,
  pub target: String,
  #[serde(rename = "controlKey")]
  pub control_key: String,
  pub link: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditLinkResult {
  #[serde(rename = "linkId")]
  pub link_id: String,
  pub target: String,
  pub link: String
}