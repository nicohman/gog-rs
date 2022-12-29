use gog::*;
use serde_json::value::Value;
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
    pub dlcs: Vec<GameDetailsP>,
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
    pub fn into_details(self, down: Downloads) -> GameDetails {
        GameDetails {
            title: self.title,
            background_image: self.background_image,
            cd_key: self.cd_key,
            text_information: self.text_information,
            downloads: down,
            extras: self.extras,
            dlcs: self
                .dlcs
                .into_iter()
                .filter_map(|mut x| {
                    if !x.downloads.is_empty() {
                        x.downloads[0].remove(0);
                        let stri = serde_json::to_string(&x.downloads[0][0]).unwrap();
                        Some(x.into_details(serde_json::from_str(&stri).unwrap()))
                    } else {
                        None
                    }
                })
                .collect(),
            tags: self.tags,
            is_pre_order: self.is_pre_order,
            release_timestamp: self.release_timestamp,
            messages: self.messages,
            changelog: self.changelog,
            forum_link: self.forum_link,
            is_base_product_missing: self.is_base_product_missing,
            missing_base_product: self.missing_base_product,
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Success {
    pub success: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Id {
    pub id: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusDel {
    pub status: String,
}
