use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NewLink {
  #[serde(rename = "linkId")]
  pub link_id: Option<String>,
  pub target: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditLink {
  #[serde(rename = "linkId")]
  pub link_id: String,
  #[serde(rename = "newLinkId")]
  pub new_link_id: Option<String>,
  pub target: Option<String>,
  #[serde(rename = "controlKey")]
  pub control_key: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeleteLink {
  #[serde(rename = "linkId")]
  pub link_id: String,
  #[serde(rename = "controlKey")]
  pub control_key: String
}