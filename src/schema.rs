use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepositoryResult {
	#[serde(rename = "category")]
	pub category: Option<String>,

	#[serde(rename = "created")]
	pub created: Option<String>,

	#[serde(rename = "creator_id")]
	pub creator_id: Option<String>,

	#[serde(rename = "creator_name")]
	pub creator_name: Option<String>,

	#[serde(rename = "description")]
	pub description: Option<String>,

	#[serde(rename = "domain_id")]
	pub domain_id: Option<String>,

	#[serde(rename = "id")]
	pub id: Option<i32>,

	#[serde(rename = "internal_path")]
	pub internal_path: Option<String>,

	#[serde(rename = "is_public")]
	pub is_public: Option<bool>,

	#[serde(rename = "name")]
	pub name: Option<String>,

	#[serde(rename = "ns_id")]
	pub ns_id: Option<i32>,

	#[serde(rename = "num_download")]
	pub num_download: Option<i32>,

	#[serde(rename = "num_images")]
	pub num_images: Option<i32>,

	#[serde(rename = "path")]
	pub path: Option<String>,

	#[serde(rename = "priority")]
	pub priority: Option<i32>,

	#[serde(rename = "size")]
	pub size: Option<i32>,

	#[serde(rename = "updated")]
	pub updated: Option<String>,

	#[serde(rename = "url")]
	pub url: Option<String>,
}
