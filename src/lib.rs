//! This crate provides an easy interface to communicate with the not-so-easy (unofficial) GOG API.
//! Many thanks to [Yepoleb](https://github.com/Yepoleb), who made
//! [this](https://gogapidocs.readthedocs.io/en/latest/index.html) very helpful set of docs.
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate serde;
mod containers;
/// Module for GOG structs and responses
pub mod gog;
/// Module for OAuth token stuff
pub mod token;
pub mod error;
use error::*;
use connect::*;
use containers::*;
use domains::*;
use gog::*;
use product::*;
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde_json::value::{Map, Value};
use token::Token;
use std::cell::RefCell;
use ErrorKind::*;
const GET: Method = Method::GET;
const POST: Method = Method::POST;
/// This is returned from functions that GOG doesn't return anything for. Should only be used for error-checking to see if requests failed, etc.
pub type EmptyResponse = ::std::result::Result<Response, Error>;
type NResult<T, E> = ::std::result::Result<T, E>;
macro_rules! map_p {
    ($($js: tt)+) => {
        Some(json!($($js)+).as_object().unwrap().clone())
    }
}
/// The main GOG Struct that you'll use to make API calls.
pub struct Gog {
    pub token: RefCell<Token>,
    client: RefCell<Client>,
    pub auto_update: bool
}
impl Gog {
    /// Initializes out of a token from a login code
    pub fn from_login_code(code: &str) -> Gog {
        Gog::from_token(Token::from_login_code(code).unwrap())
    }
    /// Creates using a pre-made token
    pub fn new(token: Token) -> Gog {
        Gog::from_token(token)
    }
    fn from_token(token: Token) -> Gog {
        let mut headers = Gog::headers_token(&token.access_token);
        let mut client = Client::builder();
        client = client.default_headers(headers);
        return Gog {
            token: RefCell::new(token),
            client: RefCell::new(client.build().unwrap()),
            auto_update: true
        };
    }
    fn update_token(&self, token: Token) {
        let mut headers = Gog::headers_token(&token.access_token);
        let mut client = Client::builder();
        self.client.replace(client.default_headers(headers).build().unwrap());
        self.token.replace(token);
    }
    fn headers_token(at: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            ("Bearer ".to_string() + at).parse().unwrap(),
        );
        headers.insert("CSRF", "csrf=true".parse().unwrap());
        return headers;
    }
    fn rget(&self, domain: &str, path: &str, params: Option<Map<String, Value>>) -> Result<Response> {
        self.rreq(GET, domain, path, params)
    }
    fn rpost(&self, domain: &str, path: &str, params: Option<Map<String, Value>>) -> Result<Response> {
        self.rreq(POST, domain, path, params)
    }
    fn rreq(
        &self,
        method: Method,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
    ) -> Result<Response> {
        if self.token.borrow().is_expired() {
            if self.auto_update {
                self.update_token(self.token.borrow().refresh()?);
                return self.rreq(method,domain,path,params);
            } else {
            return Err(ExpiredToken.into());
        }
        } else {
            let mut url = domain.to_string() + path;
            if params.is_some() {
                let params = params.unwrap();
                if params.len() > 0 {
                    url = url + "?";
                    for (k, v) in params.iter() {
                        url = url + k + "=" + &v.to_string() + "&";
                    }
                    url.pop();
                }
            }
            Ok(self.client.borrow().request(method, &url).send()?)
        }
    }
    fn fget<T>(
        &self,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.freq(GET, domain, path, params)
    }
    fn fpost<T>(
        &self,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.freq(POST, domain, path, params)
    }

    fn freq<T>(
        &self,
        method: Method,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut res = self.rreq(method, domain, path, params)?;
        let st = res.text()?;
        Ok(serde_json::from_str(&st)?)
    }
    fn nfreq<T>(&self, method:Method, domain: &str, path: &str, params: Option<Map<String, Value>>, nested: &str) -> Result<T> where T: DeserializeOwned {
            let r : Map<String, Value> = self.freq(method, domain, path, params)?;
            if r.contains_key(nested) {
                return Ok(serde_json::from_str(&r.get(nested).unwrap().to_string())?);
            } else {
                return Err(MissingField(nested.to_string()).into());
            }
    }
    fn nfget<T>(&self, domain: &str, path: &str, params: Option<Map<String, Value>>, nested: &str) -> Result<T> where T: DeserializeOwned {
        self.nfreq(GET, domain, path, params, nested)
    }
    fn nfpost<T>(&self, domain: &str, path: &str, params: Option<Map<String, Value>>, nested: &str) -> Result<T> where T: DeserializeOwned {
        self.nfreq(POST, domain, path, params, nested)
    }

    /// Gets the data of the user that is currently logged in
    pub fn get_user_data(&self) -> Result<UserData> {
        self.fget(EMBD, "/userData.json", None)
    }
    /// Gets any publically available data about a user
    pub fn get_pub_info(&self, uid: i64, expand: Vec<String>) -> Result<PubInfo> {
        self.fget(
            EMBD,
            &("/users/info/".to_string() + &uid.to_string()),
            map_p!({
            "expand":expand.iter().try_fold("".to_string(), fold_mult).unwrap()
            }),
        )
    }
    /// Gets a user's owned games. Only gameids.
    pub fn get_games(&self) -> Result<Vec<i64>> {
        let r: OwnedGames = self.fget(EMBD, "/user/data/games", None)?;
        Ok(r.owned)
    }
    /// Gets more info about a game by gameid
    pub fn get_game_details(&self, game_id: i64) -> Result<GameDetails> {
        let mut res: GameDetailsP = self.fget(
            EMBD,
            &("/account/gameDetails/".to_string() + &game_id.to_string() + ".json"),
            None,
        )?;
            res.downloads[0].remove(0);
            let downloads: Downloads = serde_json::from_str(
                &serde_json::to_string(&res.downloads[0][0])?,
            )?;
            Ok(res.to_details(downloads))
    }
    /// Hides a product from your library
    pub fn hide_product(&self, game_id: i64) -> EmptyResponse {
        self.rget(EMBD, &("/account/hideProduct".to_string()+&game_id.to_string()), None)
    }
    /// Reveals a product in your library
    pub fn reveal_product(&self, game_id: i64) -> EmptyResponse {
        self.rget(EMBD, &("/account/revealProduct".to_string()+&game_id.to_string()), None)
    }
    /// Gets the wishlist of the current user
    pub fn wishlist(&self) -> Result<Wishlist> {
        self.fget(EMBD, "/user/wishlist.json", None)
    }
    /// Adds an item to the wishlist. Returns wishlist
    pub fn add_wishlist(&self, game_id: i64) -> Result<Wishlist> {
        self.fget(
            EMBD,
            &("/user/wishlist/add/".to_string() + &game_id.to_string()),
            None,
        )
    }
    /// Removes an item from wishlist. Returns wishlist
    pub fn rm_wishlist(&self, game_id: i64) -> Result<Wishlist> {
        self.fget(
            EMBD,
            &("/user/wishlist/remove/".to_string() + &game_id.to_string()),
            None,
        )
    }
    /// Sets birthday of account. Date should be in ISO 8601 format
    pub fn save_birthday(&self, bday: &str) -> EmptyResponse {
        self.rget(EMBD, &("/account/save_birthday".to_string()+bday), None)
    }
    /// Sets country of account. Country should be in ISO 3166 format
    pub fn save_country(&self, country: &str) -> EmptyResponse {
        self.rget(EMBD, &("/account/save_country".to_string()+country),None)
    }
    /// Changes default currency. Currency is in ISO 4217 format. Only currencies supported are
    /// ones in the currency enum.
    pub fn save_currency(&self, currency: Currency) -> EmptyResponse {
        self.rget(EMBD, &("/user/changeCurrency".to_string()+&currency.to_string()), None)
    }
    /// Changes default language. Possible languages are available as constants in the langauge
    /// enum.
    pub fn save_language(&self, language: Language) -> EmptyResponse {
        self.rget(EMBD, &("/user/changeLanguage".to_string()+&language.to_string()), None)
    }
    /// Gets info about the steam account linked to GOG Connect for the user id
    pub fn connect_account(&self, user_id: i64) -> Result<LinkedSteam> {
        self.fget(
            EMBD,
            &("/api/v1/users/".to_string() + &user_id.to_string() + "/gogLink/steam/linkedAccount"),
            None,
        )
    }
    /// Gets claimable status of currently available games on GOG Connect
    pub fn connect_status(&self, user_id: i64) -> Result<ConnectStatus> {
        self.fget(
            EMBD,
            &("/api/v1/users/".to_string()
                + &user_id.to_string()
                + "/gogLink/steam/exchangeableProducts"),
            None,
        )
    }
    /// Scans Connect for claimable games
    pub fn connect_scan(&self, user_id: i64) -> EmptyResponse {
        self.rget(EMBD, &("/api/v1/users/".to_string() + &user_id.to_string()+ "/gogLink/steam/synchronizeUserProfile"), None)
    }
    /// Claims all available Connect games
    pub fn connect_claim(&self, user_id: i64) -> EmptyResponse {
        self.rget(EMBD, &("/api/v1/users/".to_string()+ &user_id.to_string() + "/gogLink/steam/claimProducts"), None)
    }
    /// Returns detailed info about a product/products.
    pub fn product(&self, ids: Vec<i64>, expand: Vec<String>) -> Result<Vec<Product>> {
        self.fget(
            API,
            "/products",
            map_p!({
            "expand":expand.iter().try_fold("".to_string(), fold_mult).unwrap(),
            "ids": ids.iter().try_fold("".to_string(), |acc, x|{
                let done : Result<String> = Ok(acc +"," +&x.to_string());
                done
            }).unwrap()
        }),
        )
    }
    /// Get a list of achievements for a game and user id
    pub fn achievements(&self, product_id: i64, user_id: i64) -> Result<AchievementList> {
        self.fget(
            GPLAY,
            &("/clients/".to_string()
                + &product_id.to_string()
                + "/users/"
                + &user_id.to_string()
                + "/achievements"),
            None,
        )
    }
    /// Adds tag with tagid to product
    pub fn add_tag(&self, product_id: i64, tag_id: i64) -> Result<bool> {
        let res: Result<Success> = self.fget(
            EMBD,
            "/account/tags/attach",
            map_p!({
            "product_id":product_id,
            "tag_id":tag_id
        }),
        );
        res.map(|x| x.success)
    }
    /// Removes tag with tagid from product
    pub fn rm_tag(&self, product_id: i64, tag_id: i64) -> Result<bool> {
        self.nfget(
            EMBD,
            "/account/tags/detach",
            map_p!({
            "product_id":product_id,
            "tag_id":tag_id
        }),"success"
        )
    }
    /// Creates a new tag. Returns the tag's id
    pub fn create_tag(&self, name: &str) -> Result<i64> {
        return self.nfget(EMBD, "/account/tags/add", map_p!({ "name": name }), "id").map(|x: String| x.parse::<i64>().unwrap());
    }
    /// Deletes a tag. Returns bool indicating success
    pub fn delete_tag(&self, tag_id: i64) -> Result<bool> {
        let res: Result<StatusDel> =
            self.fget(EMBD, "/account/tags/delete", map_p!({ "tag_id": tag_id }));
        res.map(|x| {
            if x.status.as_str() == "deleted" {
                return true;
            } else {
                return false;
            }
        })
    }
    /// Changes newsletter subscription status
    pub fn newsletter_subscription(&self, enabled: bool) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/account/save_newsletter_subscription/".to_string()
                + &bool_to_int(enabled).to_string()),
            None,
        )
    }
    /// Changes promo subscription status
    pub fn promo_subscription(&self, enabled: bool) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/account/save_promo_subscription/".to_string() + &bool_to_int(enabled).to_string()),
            None,
        )
    }
    /// Changes wishlist subscription status
    pub fn wishlist_subscription(&self, enabled: bool) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/account/save_wishlist_notification/".to_string() + &bool_to_int(enabled).to_string()),
            None,
        )
    }
    /// Shortcut function to enable or disable all subscriptions
    pub fn all_subscription(&self, enabled:bool) -> Vec<EmptyResponse> {
        vec![self.newsletter_subscription(enabled),self.promo_subscription(enabled),self.wishlist_subscription(enabled)]
    }
    /// Gets games this user has rated
    pub fn game_ratings(&self) -> Result<Vec<(String, i64)>> {
        let g : Map<String, Value> = self.nfget(EMBD,"/user/games_rating.json", None, "games_rating")?;
        Ok(g.iter().map(|x| return (x.0.to_owned(), x.1.as_i64().unwrap())).collect::<Vec<(String, i64)>>())
    }
    /// Gets reviews the user has voted on
    pub fn voted_reviews(&self) -> Result<Vec<i64>> {
        return self.nfget(EMBD, "/user/review_votes.json", None, "reviews");
    }
    /// Reports a review
    pub fn report_review(&self, review_id: i32) -> Result<bool> {
        self.nfpost(EMBD, &("/reviews/report/review/".to_string()+&review_id.to_string()+".json"), None, "reported")
    }
    /// Sets library background style
    pub fn library_background(&self, bg: ShelfBackground) -> EmptyResponse {
       self.rpost(EMBD, &("/account/save_shelf_background/".to_string() +bg.as_str()), None)
    }
}
fn fold_mult(acc: String, now: &String) -> Result<String> {
    return Ok(acc + "," + now);
}
fn bool_to_int(b: bool) -> i32 {
    let mut par = 0;
    if b {
        par = 1;
    }
    return par;
}
