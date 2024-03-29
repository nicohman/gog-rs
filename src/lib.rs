//! This crate provides an easy interface to communicate with the not-so-easy (unofficial) GOG API.
//! Many thanks to [Yepoleb](https://github.com/Yepoleb), who made
//! [this](https://gogapidocs.readthedocs.io/en/latest/index.html) very helpful set of docs.
use serde_json::json;
mod containers;
/// Provides error-handling logic
mod error;
/// Module for extracting GOG installers into their component parts
pub mod extract;
/// Module for GOG structs and responses
pub mod gog;
/// Module for OAuth token management
pub mod token;
use connect::*;
use containers::*;
use curl::easy::{Easy2, Handler, WriteError};
use domains::*;
/// Main error for GOG calls
pub use error::Error;
pub use error::ErrorKind;
pub use error::Result;
use extract::*;
use gog::*;
use product::*;
use regex::*;
use reqwest::blocking::{Client, Response};
use reqwest::header::*;
use reqwest::redirect::Policy;
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde_json::value::{Map, Value};
use std::cell::RefCell;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use token::Token;
use ErrorKind::*;

const GET: Method = Method::GET;
const POST: Method = Method::POST;

// This is returned from functions that GOG doesn't return anything for. Should only be used for error-checking to see if requests failed, etc.
pub type EmptyResponse = ::std::result::Result<Response, Error>;
macro_rules! map_p {
    ($($js: tt)+) => {
        Some(json!($($js)+).as_object().unwrap().clone())
    }
}

// The main GOG Struct that you'll use to make API calls.
pub struct Gog {
    pub token: RefCell<Token>,
    pub client: RefCell<Client>,
    pub client_noredirect: RefCell<Client>,
    pub auto_update: bool,
}
impl Gog {
    // Initializes out of a token from a login code
    pub fn from_login_code(code: &str) -> Gog {
        Gog::from_token(Token::from_login_code(code).unwrap())
    }

    // Creates using a pre-made token
    pub fn new(token: Token) -> Gog {
        Gog::from_token(token)
    }

    fn from_token(token: Token) -> Gog {
        let headers = Gog::headers_token(&token.access_token);
        let mut client = Client::builder();
        let mut client_re = Client::builder().redirect(Policy::none());
        client = client.default_headers(headers.clone());
        client_re = client_re.default_headers(headers);
        Gog {
            token: RefCell::new(token),
            client: RefCell::new(client.build().unwrap()),
            client_noredirect: RefCell::new(client_re.build().unwrap()),
            auto_update: true,
        }
    }

    fn update_token(&self, token: Token) {
        let headers = Gog::headers_token(&token.access_token);
        let client = Client::builder();
        let client_re = Client::builder().redirect(Policy::none());
        self.client
            .replace(client.default_headers(headers.clone()).build().unwrap());
        self.client_noredirect
            .replace(client_re.default_headers(headers).build().unwrap());
        self.token.replace(token);
    }

    pub fn uid_string(&self) -> String {
        self.token.borrow().user_id.clone()
    }

    pub fn uid(&self) -> i64 {
        self.token.borrow().user_id.parse().unwrap()
    }

    fn headers_token(at: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            ("Bearer ".to_string() + at).parse().unwrap(),
        );
        // GOG now requires this magic cookie to be included in all requests.
        headers.insert("CSRF", "csrf=true".parse().unwrap());
        headers
    }

    fn rget(
        &self,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
    ) -> Result<Response> {
        self.rreq(GET, domain, path, params)
    }

    fn rpost(
        &self,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
    ) -> Result<Response> {
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
                let new_token = self.token.borrow().refresh()?;
                self.update_token(new_token);
                self.rreq(method, domain, path, params)
            } else {
                Err(ExpiredToken.into())
            }
        } else {
            let mut url = domain.to_string() + path;
            if let Some(temp_params) = params {
                let params = temp_params;
                if !params.is_empty() {
                    url += "?";
                    for (k, v) in params.iter() {
                        url = url + k + "=" + &v.to_string() + "&";
                    }
                    url.pop();
                }
            }
            Ok(self.client.borrow().request(method, &url).send()?)
        }
    }

    fn fget<T>(&self, domain: &str, path: &str, params: Option<Map<String, Value>>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.freq(GET, domain, path, params)
    }

    fn _fpost<T>(&self, domain: &str, path: &str, params: Option<Map<String, Value>>) -> Result<T>
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
        let res = self.rreq(method, domain, path, params)?;
        let st = res.text()?;
        Ok(serde_json::from_str(&st)?)
    }

    fn nfreq<T>(
        &self,
        method: Method,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
        nested: &str,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let r: Map<String, Value> = self.freq(method, domain, path, params)?;
        if r.contains_key(nested) {
            Ok(serde_json::from_str(&r.get(nested).unwrap().to_string())?)
        } else {
            Err(MissingField(nested.to_string()).into())
        }
    }

    fn nfget<T>(
        &self,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
        nested: &str,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.nfreq(GET, domain, path, params, nested)
    }

    fn nfpost<T>(
        &self,
        domain: &str,
        path: &str,
        params: Option<Map<String, Value>>,
        nested: &str,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.nfreq(POST, domain, path, params, nested)
    }

    // Gets the data of the user that is currently logged in
    pub fn get_user_data(&self) -> Result<UserData> {
        self.fget(EMBD, "/userData.json", None)
    }

    // Gets any publically available data about a user
    pub fn get_pub_info(&self, uid: i64, expand: Vec<String>) -> Result<PubInfo> {
        self.fget(
            EMBD,
            &("/users/info/".to_string() + &uid.to_string()),
            map_p!({
            "expand": expand.iter().fold("".to_string(), fold_mult)
            }),
        )
    }

    // Gets a user's owned games. Only gameids.
    pub fn get_games(&self) -> Result<Vec<i64>> {
        let r: OwnedGames = self.fget(EMBD, "/user/data/games", None)?;
        Ok(r.owned)
    }

    // Gets more info about a game by gameid
    pub fn get_game_details(&self, game_id: i64) -> Result<GameDetails> {
        let mut res: GameDetailsP = self.fget(
            EMBD,
            &("/account/gameDetails/".to_string() + &game_id.to_string() + ".json"),
            None,
        )?;
        if !res.downloads.is_empty() {
            res.downloads[0].remove(0);
            let downloads: Downloads =
                serde_json::from_str(&serde_json::to_string(&res.downloads[0][0])?)?;
            Ok(res.into_details(downloads))
        } else {
            Err(NotAvailable.into())
        }
    }

    // Returns a vec of game parts
    pub fn download_game(&self, downloads: Vec<Download>) -> Vec<Result<Response>> {
        downloads
            .iter()
            .map(|x| {
                let mut url = BASE.to_string() + &x.manual_url;
                let mut response;
                loop {
                    let temp_response = self.client_noredirect.borrow().get(url).send();
                    if let Ok(temp) = temp_response {
                        response = temp;
                        let headers = response.headers();
                        // GOG appears to be inconsistent with returning either 301/302, so this just checks for a redirect location.
                        if headers.contains_key("location") {
                            url = headers
                                .get("location")
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string();
                        } else {
                            break;
                        }
                    } else {
                        return Err(temp_response.err().unwrap().into());
                    }
                }
                Ok(response)
            })
            .collect()
    }

    // Hides a product from your library
    pub fn hide_product(&self, game_id: i64) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/account/hideProduct".to_string() + &game_id.to_string()),
            None,
        )
    }

    // Reveals a product in your library
    pub fn reveal_product(&self, game_id: i64) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/account/revealProduct".to_string() + &game_id.to_string()),
            None,
        )
    }

    // Gets the wishlist of the current user
    pub fn wishlist(&self) -> Result<Wishlist> {
        self.fget(EMBD, "/user/wishlist.json", None)
    }

    // Adds an item to the wishlist. Returns wishlist
    pub fn add_wishlist(&self, game_id: i64) -> Result<Wishlist> {
        self.fget(
            EMBD,
            &("/user/wishlist/add/".to_string() + &game_id.to_string()),
            None,
        )
    }

    // Removes an item from wishlist. Returns wishlist
    pub fn rm_wishlist(&self, game_id: i64) -> Result<Wishlist> {
        self.fget(
            EMBD,
            &("/user/wishlist/remove/".to_string() + &game_id.to_string()),
            None,
        )
    }

    // Sets birthday of account. Date should be in ISO 8601 format
    pub fn save_birthday(&self, bday: &str) -> EmptyResponse {
        self.rget(EMBD, &("/account/save_birthday".to_string() + bday), None)
    }

    // Sets country of account. Country should be in ISO 3166 format
    pub fn save_country(&self, country: &str) -> EmptyResponse {
        self.rget(EMBD, &("/account/save_country".to_string() + country), None)
    }

    // Changes default currency. Currency is in ISO 4217 format. Only currencies supported are
    // ones in the currency enum.
    pub fn save_currency(&self, currency: Currency) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/user/changeCurrency".to_string() + &currency.to_string()),
            None,
        )
    }

    // Changes default language. Possible languages are available as constants in the langauge
    // enum.
    pub fn save_language(&self, language: Language) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/user/changeLanguage".to_string() + &language.to_string()),
            None,
        )
    }

    // Gets info about the steam account linked to GOG Connect for the user id
    pub fn connect_account(&self, user_id: i64) -> Result<LinkedSteam> {
        self.fget(
            EMBD,
            &("/api/v1/users/".to_string() + &user_id.to_string() + "/gogLink/steam/linkedAccount"),
            None,
        )
    }

    // Gets claimable status of currently available games on GOG Connect
    pub fn connect_status(&self, user_id: i64) -> Result<ConnectStatus> {
        let st = self
            .rget(
                EMBD,
                &("/api/v1/users/".to_string()
                    + &user_id.to_string()
                    + "/gogLink/steam/exchangeableProducts"),
                None,
            )?
            .text()?;
        if let Ok(cs) = serde_json::from_str(&st) {
            return Ok(cs);
        } else {
            let map: Map<String, Value> = serde_json::from_str(&st)?;
            if let Some(items) = map.get("items") {
                let array = items.as_array();
                if array.is_some() && array.unwrap().is_empty() {
                    return Err(NotAvailable.into());
                }
            }
        }
        Err(MissingField("items".to_string()).into())
    }

    // Scans Connect for claimable games
    pub fn connect_scan(&self, user_id: i64) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/api/v1/users/".to_string()
                + &user_id.to_string()
                + "/gogLink/steam/synchronizeUserProfile"),
            None,
        )
    }

    // Claims all available Connect games
    pub fn connect_claim(&self, user_id: i64) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/api/v1/users/".to_string() + &user_id.to_string() + "/gogLink/steam/claimProducts"),
            None,
        )
    }

    // Returns detailed info about a product/products.
    pub fn product(&self, ids: Vec<i64>, expand: Vec<String>) -> Result<Vec<Product>> {
        self.fget(
            API,
            "/products",
            map_p!({
                "expand": expand.iter().fold("".to_string(), fold_mult),
                "ids": ids.iter().fold("".to_string(), |acc, x|{
                    acc + "," + &x.to_string()
                })
            }),
        )
    }

    // Get a list of achievements for a game and user id
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

    // Adds tag with tagid to product
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

    // Removes tag with tagid from product
    pub fn rm_tag(&self, product_id: i64, tag_id: i64) -> Result<bool> {
        self.nfget(
            EMBD,
            "/account/tags/detach",
            map_p!({
                "product_id":product_id,
                "tag_id":tag_id
            }),
            "success",
        )
    }

    // Fetches info about a set of products owned by the user based on search criteria
    pub fn get_filtered_products(&self, params: FilterParams) -> Result<FilteredProducts> {
        // GOG.com url is just to avoid "cannot be a base" url parse error, as we only need the path anyways
        let url = reqwest::Url::parse(
            &("https://gog.com/account/getFilteredProducts".to_string()
                + &params.to_query_string()),
        )
        .unwrap();
        let path = url.path().to_string() + "?" + url.query().unwrap();
        self.fget(EMBD, &path, None)
    }

    // Fetches info about all products matching criteria
    pub fn get_all_filtered_products(&self, params: FilterParams) -> Result<Vec<ProductDetails>> {
        let url = reqwest::Url::parse(
            &("https://gog.com/account/getFilteredProducts".to_string()
                + &params.to_query_string()),
        )
        .unwrap();
        let mut page = 1;
        let path = url.path().to_string() + "?" + url.query().unwrap();
        let mut products = vec![];
        loop {
            let res: FilteredProducts =
                self.fget(EMBD, &format!("{}&page={}", path, page), None)?;
            products.push(res.products);
            if page >= res.total_pages {
                break;
            } else {
                page += 1;
            }
        }
        Ok(products.into_iter().flatten().collect())
    }

    // Fetches info about a set of products based on search criteria
    pub fn get_products(&self, params: FilterParams) -> Result<Vec<UnownedProductDetails>> {
        // GOG.com url is just to avoid "cannot be a base" url parse error, as we only need the path anyways
        let url = reqwest::Url::parse(
            &("https://gog.com/games/ajax/filtered".to_string() + &params.to_query_string()),
        )
        .unwrap();
        let path = url.path().to_string() + "?" + url.query().unwrap();
        self.nfget(EMBD, &path, None, "products")
    }

    // Creates a new tag. Returns the tag's id
    pub fn create_tag(&self, name: &str) -> Result<i64> {
        return self
            .nfget(EMBD, "/account/tags/add", map_p!({ "name": name }), "id")
            .map(|x: String| x.parse::<i64>().unwrap());
    }

    // Deletes a tag. Returns bool indicating success
    pub fn delete_tag(&self, tag_id: i64) -> Result<bool> {
        let res: Result<StatusDel> =
            self.fget(EMBD, "/account/tags/delete", map_p!({ "tag_id": tag_id }));
        res.map(|x| return x.status.as_str() == "deleted")
    }

    // Changes newsletter subscription status
    pub fn newsletter_subscription(&self, enabled: bool) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/account/save_newsletter_subscription/".to_string() + &(enabled as i32).to_string()),
            None,
        )
    }

    // Changes promo subscription status
    pub fn promo_subscription(&self, enabled: bool) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/account/save_promo_subscription/".to_string() + &(enabled as i32).to_string()),
            None,
        )
    }

    // Changes wishlist subscription status
    pub fn wishlist_subscription(&self, enabled: bool) -> EmptyResponse {
        self.rget(
            EMBD,
            &("/account/save_wishlist_notification/".to_string() + &(enabled as i32).to_string()),
            None,
        )
    }

    // Shortcut function to enable or disable all subscriptions
    pub fn all_subscription(&self, enabled: bool) -> Vec<EmptyResponse> {
        vec![
            self.newsletter_subscription(enabled),
            self.promo_subscription(enabled),
            self.wishlist_subscription(enabled),
        ]
    }

    // Gets games this user has rated
    pub fn game_ratings(&self) -> Result<Vec<(String, i64)>> {
        let g: Map<String, Value> =
            self.nfget(EMBD, "/user/games_rating.json", None, "games_rating")?;
        Ok(g.iter()
            .map(|x| (x.0.to_owned(), x.1.as_i64().unwrap()))
            .collect::<Vec<(String, i64)>>())
    }

    // Gets reviews the user has voted on
    pub fn voted_reviews(&self) -> Result<Vec<i64>> {
        self.nfget(EMBD, "/user/review_votes.json", None, "reviews")
    }

    // Reports a review
    pub fn report_review(&self, review_id: i32) -> Result<bool> {
        self.nfpost(
            EMBD,
            &("/reviews/report/review/".to_string() + &review_id.to_string() + ".json"),
            None,
            "reported",
        )
    }

    // Sets library background style
    pub fn library_background(&self, bg: ShelfBackground) -> EmptyResponse {
        self.rpost(
            EMBD,
            &("/account/save_shelf_background/".to_string() + bg.as_str()),
            None,
        )
    }

    // Returns list of friends
    pub fn friends(&self) -> Result<Vec<Friend>> {
        self.nfget(
            CHAT,
            &("/users/".to_string() + &self.uid_string() + "/friends"),
            None,
            "items",
        )
    }

    fn get_sizes<R: Read>(&self, bufreader: &mut BufReader<R>) -> Result<(usize, usize)> {
        let mut buffer = String::new();
        let mut script_size = 0;
        let mut script_bytes = 0;
        let mut script = String::new();
        let mut i = 1;
        let mut filesize = 0;
        let filesize_reg = Regex::new(r#"filesizes="(\d+)"#).unwrap();
        let offset_reg = Regex::new(r"offset=`head -n (\d+)").unwrap();
        loop {
            let read = bufreader.read_line(&mut buffer).unwrap();
            script_bytes += read;
            if script_size != 0 && script_size > i {
                script += &buffer;
            } else if script_size != 0 && script_size <= i && filesize != 0 {
                break;
            }
            if script_size == 0 {
                if let Some(captures) = offset_reg.captures(&buffer) {
                    if captures.len() > 1 {
                        script_size = captures[1].to_string().parse().unwrap();
                    }
                }
            }
            if filesize == 0 {
                if let Some(captures) = filesize_reg.captures(&buffer) {
                    if captures.len() > 1 {
                        filesize = captures[1].to_string().parse().unwrap();
                    }
                }
            }
            i += 1;
        }
        Ok((script_bytes, filesize))
    }

    // Downloads a file partially, using only access token instead of the full Gog struct
    pub fn download_request_range_at<H: Handler>(
        at: impl Into<String>,
        url: impl Into<String>,
        handler: H,
        start: i64,
        end: i64,
    ) -> Result<Easy2<H>> {
        let url = url.into();
        let mut easy = Easy2::new(handler);
        easy.url(&url)?;
        easy.range(&format!("{}-{}", start, end))?;
        easy.follow_location(true)?;
        let mut list = curl::easy::List::new();
        list.append("CSRF: true")?;
        list.append(&format!("Authentication: Bearer {}", at.into()))?;
        easy.get(true)?;
        easy.http_headers(list)?;
        easy.perform()?;
        Ok(easy)
    }

    // Downloads a file partially
    pub fn download_request_range(
        &self,
        url: impl Into<String>,
        start: i64,
        end: i64,
    ) -> Result<Vec<u8>> {
        Ok(Gog::download_request_range_at(
            self.token.borrow().access_token.as_str(),
            url,
            Collector(Vec::new()),
            start,
            end,
        )?
        .get_ref()
        .0
        .clone())
    }

    // Extracts data on downloads
    pub fn extract_data(&self, downloads: Vec<Download>) -> Result<Vec<ZipData>> {
        let mut zips = vec![];
        let mut responses = self.download_game(downloads.clone());
        for down in downloads {
            let mut url = BASE.to_string() + &down.manual_url;
            let mut response;
            loop {
                if let Ok(temp_response) = self.client_noredirect.borrow().get(&url).send() {
                    response = temp_response;
                    let headers = response.headers();
                    // GOG appears to be inconsistent with returning either 301/302,
                    // so this just checks for a redirect location.
                    if headers.contains_key("location") {
                        url = headers
                            .get("location")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();
                    } else {
                        break;
                    }
                }
            }
            let response = responses.remove(0).expect("Couldn't get download");
            let size = response
                .headers()
                .get(CONTENT_LENGTH)
                .unwrap()
                .to_str()
                .expect("Couldn't convert to string")
                .parse()
                .unwrap();
            let mut bufreader = BufReader::new(response);
            let sizes = self.get_sizes(&mut bufreader)?;
            let eocd_offset = self.get_eocd_offset(&url, size)?;
            let off = match eocd_offset {
                EOCDOffset::Offset(offset) => offset,
                EOCDOffset::Offset64(offset) => offset,
            };
            let cd_offset;
            let records;
            let cd_size;
            let central_directory = self.download_request_range(url.as_str(), off as i64, size)?;
            let mut cd_slice = central_directory.as_slice();
            let mut cd_reader = BufReader::new(&mut cd_slice);
            match eocd_offset {
                EOCDOffset::Offset(..) => {
                    let cd = CentralDirectory::from_reader(&mut cd_reader);
                    cd_offset = cd.cd_start_offset as u64;
                    records = cd.total_cd_records as u64;
                    cd_size = cd.cd_size as u64;
                }
                EOCDOffset::Offset64(..) => {
                    let cd = CentralDirectory64::from_reader(&mut cd_reader);
                    cd_offset = cd.cd_start as u64;
                    records = cd.cd_total;
                    cd_size = cd.cd_size;
                }
            };
            let offset_beg = sizes.0 + sizes.1 + cd_offset as usize;
            let cd = self
                .download_request_range(
                    url.as_str(),
                    offset_beg as i64,
                    (offset_beg + cd_size as usize) as i64,
                )
                .unwrap();
            let mut slice = cd.as_slice();
            let mut full_reader = BufReader::new(&mut slice);
            let mut files = vec![];
            for _ in 0..records {
                let mut entry = CDEntry::from_reader(&mut full_reader);
                entry.start_offset = (sizes.0 + sizes.1) as u64 + entry.disk_offset.unwrap();
                files.push(entry);
            }
            let len = files.len();
            files[len - 1].end_offset = offset_beg as u64 - 1;
            for i in 0..(len - 1) {
                files[i].end_offset = files[i + 1].start_offset;
            }
            zips.push(ZipData {
                sizes,
                files,
                url,
                cd: None,
            });
        }
        Ok(zips)
    }

    // Gets the EOCD offset from an url
    fn get_eocd_offset(&self, url: &str, size: i64) -> Result<EOCDOffset> {
        let signature = 0x06054b50;
        let signature_64 = 0x06064b50;
        let mut offset;
        for i in 4..size + 1 {
            let pos = size - i;
            let resp = self.download_request_range(url, pos, pos + 4)?;
            let cur = pos + 4;
            let inte = vec_to_u32(&resp);
            if inte == signature {
                offset = cur;
                offset -= 4;
                return Ok(EOCDOffset::Offset(offset as usize));
            } else if inte == signature_64 {
                offset = cur;
                offset -= 4;
                return Ok(EOCDOffset::Offset64(offset as usize));
            }
        }
        Err(NotAvailable.into())
    }
}

fn fold_mult(acc: String, now: &String) -> String {
    acc + "," + now
}

fn vec_to_u32(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

// A simple curl handler for a vector of bytes
pub struct Collector(pub Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> std::result::Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}
