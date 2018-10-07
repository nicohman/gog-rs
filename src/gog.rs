use reqwest;
use serde;
use serde_json;
use serde_json::value::{Map, Value};
use serde::Deserializer;
use serde::de::Deserialize;
use std::fmt;
use containers::*;
use std::marker::PhantomData;
use serde::de::{Visitor, MapAccess};
use std::collections::{btree_map, BTreeMap};
type GMap<K, V> = BTreeMap<K,V>;
/// The domains that the API requests will be made to
pub mod domains {
    pub static API: &str = "https://api.gog.com";
    pub static CFG: &str = "https://cfg.gog.com";
    pub static CHAT: &str = "https://chat.gog.com";
    pub static CSYS: &str = "https://content-system.gog.com";
    pub static CDN: &str = "https://cdn.gog.com";
    pub static GPLAY: &str = "https://gameplay.gog.com";
    pub static PRES: &str = "https://presence.gog.com";
    pub static USRS: &str = "https://users.gog.com";
    pub static EMBD: &str = "https://embed.gog.com";
    pub static AUTH: &str = "https://auth.gog.com";
}
/// Available currencies
#[derive(Serialize, Deserialize, Debug)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    AUD,
    RUB,
    PLN,
    CAD,
    CHF,
    NOK,
    SEK,
    DKK,
}
impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
/// Available languages
#[derive(Serialize, Deserialize, Debug)]
pub enum Language {
    /// en-US
    ENUS,
    /// fr-FR
    FR,
    /// pt-BR
    PTBR,
    /// ru-RU
    RU,
    /// de-DE
    DE,
    /// zh-HANS
    ZH,
}
impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match (&self) {
            ENUS => "en-US",
            FR => "fr-FR",
            PTBR => "pt-BR",
            RU => "ru-RU",
            DE => "de-DE",
            ZH => "zh-HANS",
            // Sorry
            _ => "en-US",
        };
        write!(f, "{}", res)
    }
}
/// Statuses from get_pub_info
pub mod status {
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct FriendStatus {
        pub id: String,
        /// 0 is no friend status, 1 is having sent a friend request to this user, 2 is them having
        ///   sent you a friend request, and 3 is currently being friends
        pub status: i32,
        pub date_created: Option<String>,
        pub date_accepted: Option<String>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct WishlistStatus {
        /// 0 is private, 1 is public, and 2 is for friends only
        pub sharing: i32,
        pub url: String,
    }
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct BlockedStatus {
        pub blocked: bool,
    }
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ChatStatus {
        pub url: String,
        pub is_chat_restricted: bool,
    }
}
/// GOG Connect-related structs
pub mod connect {
    use gog::GMap;
    use serde_json::value::{Map, Value};    
    /// A GOG Connect-linked steam account
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct LinkedSteam {
        pub href: String,
        pub user: SteamUser,
        pub exchangable_steam_products: String,
    }
    /// Actual info on the steam account
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SteamUser {
        pub status: String,
        pub gog_username: String,
        pub gog_avatar: String,
        pub steam_username: String,
        pub steam_avatar: String,
    }
    /// GOG Connect games status. Items' key is a gameid.
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ConnectStatus {
        pub href: String,
        pub items: GMap<String, ConnectGame>,
    }
    ///A GOG Connect game
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ConnectGame {
        pub id: i64,
        pub status: ConnectGameStatus,
    }
    /// The status of a GOG Connect game
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub enum ConnectGameStatus {
        IMPORTED,
        READY_TO_LINK,
        UNAVAILABLE,
    }
}
/// An user's avatar urls
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Avatars {
    pub small: String,
    pub small2x: String,
    pub medium: String,
    pub medium2x: String,
    pub large: String,
    pub large2x: String,
}
use status::*;
///Data on the currently logged-in user
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub country: String,
    pub currencies: Vec<CurrencyInfo>,
    pub selected_currency: CurrencyInfo,
    pub preferred_language: LanguageInfo,
    pub rating_brand: String,
    pub is_logged_in: bool,
    pub checksum: Checksum,
    pub updates: Updates,
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub personalized_product_prices: Vec<Map<String, Value>>,
    pub personalized_series_prices: Vec<Map<String, Value>>,
}
/// Information on a currency
#[derive(Serialize, Deserialize, Debug)]
pub struct CurrencyInfo {
    pub code: Currency,
    pub symbol: String,
}
/// Informationn on a language
#[derive(Serialize, Deserialize, Debug)]
pub struct LanguageInfo {
    pub code: String,
    pub name: String,
}
/// The checksums of various user data
#[derive(Serialize, Deserialize, Debug)]
pub struct Checksum {
    pub cart: Option<String>,
    pub games: Option<String>,
    pub wishlist: Option<String>,
    pub reviews_votes: Option<String>,
    pub games_rating: Option<String>,
}
/// Waiting notifications for a user
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Updates {
    pub messages: Option<i32>,
    pub pending_friend_requests: Option<i32>,
    pub unread_chat_messages: Option<i32>,
    pub products: Option<i32>,
    pub forum: Option<i32>,
    pub total: Option<i32>,
}
/// The Different types of error
#[derive(Serialize, Deserialize, Debug)]
pub enum ErrorType {
    Gog,
    Req,
}
///An Error from an API Call. Can either be an error on reqwest's side, or Gog
#[derive(Debug)]
pub struct Error {
    pub etype: ErrorType,
    pub msg: Option<String>,
    pub error: Option<reqwest::Error>,
}
impl Error {
    /// Checks if an error comes from reqwest
    pub fn is_req(&self) -> bool {
        match self.etype {
            ErrorType::Gog => true,
            _ => false,
        }
    }
    /// Checks if an error comes from GOG
    pub fn is_gog(&self) -> bool {
        match self.etype {
            ErrorType::Req => true,
            _ => false,
        }
    }
}
/// Publically available info about an user
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PubInfo {
    pub id: String,
    pub username: String,
    pub user_since: i64,
    pub avatars: Option<Avatars>,
    pub friend_status: FriendStatus,
    pub wishlist_status: status::WishlistStatus,
    pub blocked_status: Option<status::BlockedStatus>,
    pub chat_status: Option<status::ChatStatus>,
}
/// All of the details of a specific game
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameDetails {
    pub title: String,
    pub background_image: String,
    pub cd_key: Option<String>,
    pub text_information: String,
    pub downloads: Downloads,
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
/// An extra that comes with a game, like wallpapers or soundtrack
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub manual_url: String,
    pub downloader_url: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_e: String,
    pub info: i64,
    pub size: String,
}
/// A 'tag' on a game like Favorite
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub product_count: String,
}
/// A set of builds for a game for different OSes
#[derive(Serialize, Deserialize, Debug)]
pub struct Downloads {
    pub windows: Option<Vec<Download>>,
    pub mac: Option<Vec<Download>>,
    pub linux: Option<Vec<Download>>,
}
/// Information on an available build of a game
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub manual_url: String,
    pub downloader_url: String,
    pub name: String,
    pub version: Option<String>,
    pub date: String,
    pub size: String,
}
/// A user's wishlist
#[derive(Serialize, Deserialize, Debug)]
pub struct Wishlist {
    /// Keys in wishlist are the game ids of wishlisted games
    pub wishlist: GMap<String, bool>,
    pub checksum: String,
}

