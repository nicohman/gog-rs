use serde_json::value::Map;
use serde_json::value::Value;

#[derive(Serialize, Deserialize, Debug)] 
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub country:String,
    pub currencies:Vec<Currency>,
    pub selected_currency:Currency,
    pub preferred_language:Language,
    pub rating_brand: String,
    pub is_logged_in:bool,
    pub checksum: Checksum,
    pub updates:Updates,
    pub user_id: String,
    pub username:String,
    pub email:String,
    pub personalized_product_prices:Vec<Map<String,Value>>,
    pub personalized_series_prices:Vec<Map<String, Value>>
}
#[derive(Serialize, Deserialize, Debug)] 
pub struct Currency {
    pub code:String,
    pub symbol:String
}
#[derive(Serialize, Deserialize, Debug)] 
pub struct Language {
    pub code:String,
    pub name:String
}
#[derive(Serialize, Deserialize, Debug)] 
pub struct Checksum {
    pub cart:Option<String>,
    pub games:Option<String>,
    pub wishlist:Option<String>,
    pub reviews_votes: Option<String>,
    pub games_rating: Option<String>
}
#[derive(Serialize, Deserialize, Debug)] 
#[serde(rename_all = "camelCase")]
pub struct Updates {
    pub messages:Option<i32>,
    pub pending_friend_requests:Option<i32>,
    pub unread_chat_messages:Option<i32>,
    pub products: Option<i32>,
    pub forum: Option<i32>,
    pub total: Option<i32>
}
