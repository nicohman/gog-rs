//! This crate provides an easy interface to communicate with the not-so-easy (unofficial) GOG API.
//! Many thanks to [Yepoleb](https://github.com/Yepoleb), who made
//! [this](https://gogapidocs.readthedocs.io/en/latest/index.html) very helpful set of docs.
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate reqwest;
extern crate serde;
mod containers;
/// Module for GOG structs and responses
pub mod gog;
/// Module for OAuth token stuff
pub mod token;
use connect::*;
use containers::*;
use domains::*;
use gog::*;
use product::*;
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde_json::value::{Map, Value};
use std::result::Result;
use token::Token;
use ErrorType::*;
const GET: Method = Method::GET;
const POST: Method = Method::POST;
macro_rules! map_p {
    ($($js: tt)+) => {
        Some(json!($($js)+).as_object().unwrap().clone())
    }
}
/// The main GOG Struct that you'll use to make API calls.
pub struct Gog {
    pub token: Token,
    client: Client,
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
            token: token,
            client: client.build().unwrap(),
        };
    }
    fn update_token(&mut self, token: Token) {
        let mut headers = Gog::headers_token(&token.access_token);
        let mut client = Client::builder();
        self.client = client.default_headers(headers).build().unwrap();
        self.token = token;
    }
    fn headers_token(at: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            ("Bearer ".to_string() + at).parse().unwrap(),
        );
        return headers;
    }
    fn rreq(
        &self,
        method: Method,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
    ) -> Result<Response, Error> {
        if self.token.is_expired() {
            return Err(Error {
                etype: RefreshToken,
                error: None,
                msg: None,
            });
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
            let res = self.client.request(method, &url).send();
            if res.is_err() {
                return Err(Error {
                    etype: Req,
                    error: Some(res.err().unwrap()),
                    msg: None,
                });
            } else {
                return Ok(res.unwrap());
            }
        }
    }
    fn fget<T>(
        &self,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
    ) -> Result<T, Error>
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
    ) -> Result<T, Error>
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
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let res = self.rreq(method, domain, path, params);
        if res.is_err() {
            return Err(res.err().unwrap());
        } else {
            let st = res.unwrap().text().unwrap();
            let try: Result<T, serde_json::Error> = serde_json::from_str(&st);
            if try.is_ok() {
                return Ok(try.unwrap());
            } else {
                return Err(Error {
                    etype: Gog,
                    msg: Some(format!("{:?}\n{}", try.err().unwrap(), st)),
                    error: None,
                });
            }
        }
    }
    /// Gets the data of the user that is currently logged in
    pub fn get_user_data(&self) -> Result<UserData, Error> {
        self.fget(EMBD, "/userData.json", None)
    }
    /// Gets any publically available data about a user
    pub fn get_pub_info(&self, uid: i64, expand: Vec<String>) -> Result<PubInfo, Error> {
        let r: Result<PubInfo, Error> = self.fget(
            EMBD,
            &("/users/info/".to_string() + &uid.to_string()),
            map_p!({
            "expand":expand.iter().try_fold("".to_string(), fold_mult).unwrap()
        }),
        );
        r
    }
    /// Gets a user's owned games. Only gameids.
    pub fn get_games(&self) -> Result<Vec<i64>, Error> {
        let r: Result<OwnedGames, Error> = self.fget(EMBD, "/user/data/games", None);
        if r.is_ok() {
            return Ok(r.unwrap().owned);
        } else {
            return Err(r.err().unwrap());
        }
    }
    /// Gets more info about a game by gameid
    pub fn get_game_details(&self, game_id: i64) -> Result<GameDetails, Error> {
        let r: Result<GameDetailsP, Error> = self.fget(
            EMBD,
            &("/account/gameDetails/".to_string() + &game_id.to_string() + ".json"),
            None,
        );
        if r.is_ok() {
            let mut res = r.unwrap();
            res.downloads[0].remove(0);
            let downloads: Downloads = serde_json::from_str(
                &serde_json::to_string(&res.downloads[0][0]).unwrap(),
            ).unwrap();
            Ok(res.to_details(downloads))
        } else {
            return Err(r.err().unwrap());
        }
    }
    /// Hides a product from your library
    pub fn hide_product(&self, game_id: i64) {
        self.client
            .get(&(EMBD.to_string() + "/account/hideProduct/" + &game_id.to_string()))
            .send();
    }
    /// Reveals a product in your library
    pub fn reveal_product(&self, game_id: i64) {
        self.client
            .get(&(EMBD.to_string() + "/account/revealProduct" + &game_id.to_string()))
            .send();
    }
    /// Gets the wishlist of the current user
    pub fn wishlist(&self) -> Result<Wishlist, Error> {
        self.fget(EMBD, "/user/wishlist.json", None)
    }
    /// Adds an item to the wishlist. Returns wishlist
    pub fn add_wishlist(&self, game_id: i64) -> Result<Wishlist, Error> {
        self.fget(
            EMBD,
            &("/user/wishlist/add/".to_string() + &game_id.to_string()),
            None,
        )
    }
    /// Removes an item from wishlist. Returns wishlist
    pub fn rm_wishlist(&self, game_id: i64) -> Result<Wishlist, Error> {
        self.fget(
            EMBD,
            &("/user/wishlist/remove/".to_string() + &game_id.to_string()),
            None,
        )
    }
    /// Sets birthday of account. Date should be in ISO 8601 format
    pub fn save_birthday(&self, bday: &str) {
        self.client
            .get(&(EMBD.to_string() + "/account/save_birthday/" + bday))
            .send();
    }
    /// Sets country of account. Country should be in ISO 3166 format
    pub fn save_country(&self, country: &str) {
        self.client
            .get(&(EMBD.to_string() + "/account/save_country/" + country))
            .send();
    }
    /// Changes default currency. Currency is in ISO 4217 format. Only currencies supported are
    /// ones in the currency enum.
    pub fn save_currency(&self, currency: Currency) {
        self.client
            .get(&(EMBD.to_string() + "/user/changeCurrency/" + &currency.to_string()))
            .send();
    }
    /// Changes default language. Possible languages are available as constants in the langauge
    /// enum.
    pub fn save_language(&self, language: Language) {
        self.client
            .get(&(EMBD.to_string() + "/user/changeLanguage/" + &language.to_string()))
            .send();
    }
    /// Gets info about the steam account linked to GOG Connect for the user id
    pub fn connect_account(&self, user_id: i64) -> Result<LinkedSteam, Error> {
        self.fget(
            EMBD,
            &("/api/v1/users/".to_string() + &user_id.to_string() + "/gogLink/steam/linkedAccount"),
            None,
        )
    }
    /// Gets claimable status of currently available games on GOG Connect
    pub fn connect_status(&self, user_id: i64) -> Result<ConnectStatus, Error> {
        self.fget(
            EMBD,
            &("/api/v1/users/".to_string()
                + &user_id.to_string()
                + "/gogLink/steam/exchangeableProducts"),
            None,
        )
    }
    /// Scans Connect for claimable games
    pub fn connect_scan(&self, user_id: i64) {
        self.client
            .get(
                &(EMBD.to_string()
                    + "/api/v1/users/"
                    + &user_id.to_string()
                    + "/gogLink/steam/synchronizeUserProfile"),
            )
            .send();
    }
    /// Claims all available Connect games
    pub fn connect_claim(&self, user_id: i64) {
        self.client
            .get(
                &(EMBD.to_string()
                    + "/api/v1/users/"
                    + &user_id.to_string()
                    + "/gogLink/steam/claimProducts"),
            )
            .send();
    }
    /// Returns detailed info about a product/products.
    pub fn product(&self, ids: Vec<i64>, expand: Vec<String>) -> Result<Vec<Product>, Error> {
        let r: Result<Vec<Product>, Error> = self.fget(
            API,
            "/products",
            map_p!({
            "expand":expand.iter().try_fold("".to_string(), fold_mult).unwrap(),
            "ids": ids.iter().try_fold("".to_string(), |acc, x|{
                let done : Result<String, Error> = Ok(acc +"," +&x.to_string());
                done
            }).unwrap()
        }),
        );
        r
    }
    /// Get a list of achievements for a game and user id
    pub fn achievements(&self, product_id: i64, user_id: i64) -> Result<AchievementList, Error> {
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
    pub fn add_tag(&self, product_id: i64, tag_id: i64) -> Result<bool, Error> {
        let res: Result<Success, Error> = self.fget(
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
    pub fn rm_tag(&self, product_id: i64, tag_id: i64) -> Result<bool, Error> {
        let res: Result<Success, Error> = self.fget(
            EMBD,
            "/account/tags/detach",
            map_p!({
            "product_id":product_id,
            "tag_id":tag_id
        }),
        );
        res.map(|x| x.success)
    }
    /// Creates a new tag. Returns the tag's id
    pub fn create_tag(&self, name: &str) -> Result<i64, Error> {
        let res : Result<Id, Error> = self.fget(EMBD, "/account/tags/add", map_p!({
            "name": name
        }));
        res.map(|x| x.id.parse::<i64>().unwrap())
    }
    /// Deletes a tag. Returns bool indicating success
    pub fn delete_tag(&self, tag_id: i64) -> Result<bool, Error> {
        let res : Result<StatusDel, Error> = self.fget(EMBD, "/account/tags/delete", map_p!({
            "tag_id":tag_id
        }));
        res.map(|x| {
            if x.status.as_str() == "deleted" {
                return true;
            } else {
                return false;
            }
        })
    }
}
fn fold_mult(acc: String, now: &String) -> Result<String, Error> {
    return Ok(acc + "," + now);
}
