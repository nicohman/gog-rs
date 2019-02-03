use serde_json::value::{Map, Value};
use std::collections::BTreeMap;
use std::fmt;
type GMap<K, V> = BTreeMap<K, V>;
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
    pub static BASE: &str = "https://gog.com";
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum OS {
    Linux,
    Windows,
    MacOS,
}
impl OS {
    fn codes(&self) -> String {
        match self {
            OS::Linux => "1024,2048",
            OS::Windows => "1,2,4,8,4096",
            OS::MacOS => "16,32",
        }
        .to_string()
    }
}
/// Available currencies
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
/// Shelf background styles
pub enum ShelfBackground {
    Wood,
    Mate_Black,
    Glass,
    Chrome,
    White,
    Piano_Black,
}
impl ShelfBackground {
    pub fn as_str(&self) -> &str {
        use ShelfBackground::*;
        match &self {
            Wood => "wood",
            Mate_Black => "mate_black",
            Glass => "glass",
            Chrome => "chrome",
            White => "white",
            Piano_Black => "piano_black",
        }
    }
}
impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match &self {
            Language::ENUS => "en-US",
            Language::FR => "fr-FR",
            Language::PTBR => "pt-BR",
            Language::RU => "ru-RU",
            Language::DE => "de-DE",
            Language::ZH => "zh-HANS",
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
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum ConnectGameStatus {
        IMPORTED,
        READY_TO_LINK,
        UNAVAILABLE,
    }
}
/// Things associated with the /products endpoinnt
pub mod product {
    use serde_json::value::{Map, Value};
    /// The main product struct
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Product {
        pub id: i64,
        pub title: String,
        #[serde(default)]
        pub purchase_link: Option<String>,
        pub slug: String,
        pub content_system_compatibility: OSSupport,
        pub languages: Map<String, Value>,
        pub links: Links,
        pub in_development: InDev,
        pub is_secret: bool,
        pub game_type: String,
        pub is_pre_order: bool,
        pub release_date: Option<String>,
        pub images: Images,
        pub dlcs: Value,
        pub downloads: Option<DownObject>,
        pub expanded_dlcs: Option<Vec<Value>>,
        pub description: Option<Description>,
        pub screenshots: Option<Vec<Screenshot>>,
        pub videos: Option<Value>,
        pub related_products: Option<Value>,
        pub changelog: Option<Value>,
    }

    /// What OS' this product supports
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct OSSupport {
        pub windows: bool,
        pub osx: bool,
        pub linux: bool,
    }
    /// Various links to things related to a game
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Links {
        pub purchase_link: String,
        pub product_card: String,
        pub support: String,
        pub forum: String,
    }
    /// Whether or not a game is currently being developed
    #[derive(Serialize, Deserialize, Debug)]
    pub struct InDev {
        pub active: bool,
        pub until: Option<String>,
    }
    /// Logos/Icons for a game
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Images {
        pub background: String,
        pub logo: String,
        pub logo2x: String,
        pub icon: String,
        pub sidebar_icon: String,
        pub sidebar_icon2x: String,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct DownObject {
        pub installers: Vec<Installer>,
    }
    /// An installer & its info
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Installer {
        pub id: String,
        pub name: String,
        pub os: String,
        pub language: String,
        pub language_full: String,
        pub version: Option<Value>,
        pub total_size: i64,
        pub files: Vec<File>,
    }
    /// A specific downloadable file
    #[derive(Serialize, Deserialize, Debug)]
    pub struct File {
        pub id: String,
        pub size: i64,
        pub downlink: String,
    }
    /// Descriptions of a game
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Description {
        pub lead: String,
        pub full: String,
        pub whats_cool_about_it: String,
    }
    /// A specific developer-provided screenshot
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Screenshot {
        pub image_id: String,
        pub formatter_template_url: String,
        pub formatted_images: Vec<FormattedImage>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct FormattedImage {
        pub formatter_name: String,
        pub image_url: String,
    }
}
/// Information about a friend
#[derive(Serialize, Deserialize, Debug)]
pub struct Friend {
    pub user_id: String,
    pub username: String,
    pub is_employee: bool,
}
/// Returned as part of Friends endpoint. Contains URLs to friend's avatar
#[derive(Serialize, Deserialize, Debug)]
pub struct MediumImages {
    pub medium: String,
    pub medium_2x: String,
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
/// Information on a language
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
    pub dlcs: Vec<GameDetails>,
    pub tags: Vec<Tag>,
    pub is_pre_order: bool,
    pub release_timestamp: i64,
    pub messages: Vec<Value>,
    pub changelog: Option<String>,
    pub forum_link: String,
    pub is_base_product_missing: bool,
    pub missing_base_product: Option<Value>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct FilterParams {
    pub params: Vec<FilterParam>,
}
impl FilterParams {
    pub fn from_vec(p: Vec<FilterParam>) -> FilterParams {
        return FilterParams { params: p };
    }
    pub fn from_one(p: FilterParam) -> FilterParams {
        return FilterParams { params: vec![p] };
    }
    pub fn to_query_string(&self) -> String {
        let mut s = String::from("?");
        for p in &self.params {
            s = s + p.to_string().as_str() + "&";
        }
        s.pop();
        return s;
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum FilterParam {
    MediaType(i32),
    OS(OS),
    Search(String),
}
impl FilterParam {
    pub fn to_string(&self) -> String {
        use FilterParam::*;
        match self {
            MediaType(id) => format!("mediaType={}", id),
            // OS filtering only for games, so forces games
            OS(os) => format!("system={}&mediaType=1", os.codes()),
            Search(st) => format!("search={}&mediaType=1", st),
        }
    }
}
/// Details of a product, returned from get_filtered_products
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductDetails {
    pub title: String,
    pub is_galaxy_compatible: bool,
    pub id: i64,
    pub image: String,
    pub url: String,
    pub works_on: WorksOn,
    pub category: String,
    pub rating: i32,
    pub is_coming_soon: bool,
    pub is_movie: bool,
    pub is_game: bool,
    pub slug: String,
    pub updates: i64,
    pub is_new: bool,
    pub is_hidden: bool,
}
/// Details of a product, returned from get_products
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnownedProductDetails {
    pub price: Price,
    pub is_discounted: bool,
    pub is_in_development: bool,
    pub id: i64,
    pub release_date: i64,
    pub availability: Availability,
    pub buyable: bool,
    pub sales_visibility: SalesVisibility,
    pub title: String,
    pub image: String,
    pub url: String,
    pub support_url: String,
    pub forum_url: String,
    pub works_on: WorksOn,
    pub category: String,
    pub original_category: String,
    pub rating: i64,
    #[serde(rename = "type")]
    pub product_type: i64,
    pub is_coming_soon: bool,
    pub is_price_visible: bool,
    pub is_movie: bool,
    pub is_game: bool,
    pub slug: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub amount: String,
    pub base_amount: String,
    pub final_amount: String,
    pub is_discounted: bool,
    pub discount_percentage: i64,
    pub discount_difference: String,
    pub symbol: String,
    pub is_free: bool,
    pub discount: i64,
    pub is_bonus_store_credit_included: bool,
    pub bonus_store_credit_amount: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    pub is_available: bool,
    pub is_available_in_account: bool,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SalesVisibility {
    pub is_active: bool,
    pub from_object: DurationEnd,
    pub to_object: DurationEnd,
    pub from: i64,
    pub to: i64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DurationEnd {
    pub date: String,
    pub timezone_type: i64,
    pub timezone: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorksOn {
    pub Windows: bool,
    pub Linux: bool,
    pub Mac: bool,
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
/// A list of achivements from GOG
#[derive(Serialize, Deserialize, Debug)]
pub struct AchievementList {
    pub total_count: i32,
    pub items: Vec<Achievement>,
}
/// A GOG Galaxy Achievement
#[derive(Serialize, Deserialize, Debug)]
pub struct Achievement {
    pub achievement_id: String,
    pub achievement_key: String,
    pub visible: bool,
    pub name: String,
    pub description: String,
    pub image_url_unlocked: String,
    pub image_url_locked: String,
    pub date_unlocked: Option<String>,
}
