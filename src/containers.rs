use reqwest;
use serde;
use serde_json;
use serde_json::value::{Map, Value};
use serde::Deserializer;
use serde::de::Deserialize;
use std::fmt;
use gog::*;
#[derive(Serialize, Deserialize, Debug)]
pub struct OwnedGames {
    pub owned: Vec<i64>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameDetailsP {
    pub title: String,
    pub background_image: String,
    pub cd_key: Option<String>,
    pub text_information: String,
    pub downloads: Vec<Vec<Value>>,
    pub extras: Vec<Extra>,
    pub dlcs: Value,
    pub tags: Vec<Tag>,
    pub is_pre_order: bool,
    pub release_timestamp: i64,
    pub messages: Vec<Value>,
    pub changelog: Option<String>,
    pub forum_link: String,
    pub is_base_product_missing: bool,
    pub missing_base_product: Option<Value>,
}
impl GameDetailsP {
    // Yes, this is bad. Yes, I am sorry.
    pub fn to_details(&mut self, down: Downloads) -> GameDetails {
        return GameDetails {
            title: self.title.clone(),
            background_image: self.background_image.clone(),
            cd_key: self.cd_key.clone(),
            text_information: self.text_information.clone(),
            downloads: down,
            extras: self.extras.clone(),
            dlcs: self.dlcs.clone(),
            tags: self.tags.clone(),
            is_pre_order: self.is_pre_order.clone(),
            release_timestamp: self.release_timestamp.clone(),
            messages: self.messages.clone(),
            changelog: self.changelog.clone(),
            forum_link: self.forum_link.clone(),
            is_base_product_missing: self.is_base_product_missing.clone(),
            missing_base_product: self.missing_base_product.clone(),
        };
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Success {
    pub success : bool
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Id {
    pub id: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusDel {
    pub status: String
}
