use error::{ErrorKind::*, *};
use regex::*;
use reqwest;
use select::{document::*, predicate::*};
use serde_json;
use std::time::SystemTime;
use user_agent::*;
fn convert_rsession(err: ::user_agent::ReqwestSessionError) -> crate::error::Error {
    ErrorKind::SessionNetwork(err).into()
}
/// An OAuth token. Will usually expire after an hour.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    /// How many seconds before this token expires
    pub expires_in: u64,
    pub scope: String,
    pub token_type: String,
    pub access_token: String,
    /// The uid of the user this token corresponds to
    pub user_id: String,
    /// The token that refresh uses
    pub refresh_token: String,
    pub session_id: String,
    #[serde(default = "cur_date")]
    pub updated_at: u64,
}
fn cur_date() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time is before Unix Epoch")
        .as_secs()
}
impl Token {
    /// Creates a token from a response from /token
    pub fn from_response(response: impl Into<String>) -> Result<Token> {
        Ok(serde_json::from_str(response.into().as_str())?)
    }
    /// Fetches a token using a login code
    pub fn from_login_code(code: impl Into<String>) -> Result<Token> {
        let mut res = reqwest::get(&("https://auth.gog.com/token?client_id=46899977096215655&client_secret=9d85c43b1482497dbbce61f6e4aa173a433796eeae2ca8c5f6129f2dc4de46d9&grant_type=authorization_code&redirect_uri=https%3A%2F%2Fembed.gog.com%2Fon_login_success%3Forigin%3Dclient&layout=client2&code=".to_string()+&code.into()+""))?;
        Token::from_response(res.text()?)
    }
    /// Checks if token has expired
    pub fn is_expired(&self) -> bool {
        self.updated_at + self.expires_in - cur_date() <= 0
    }
    /// Attempts to fetch an updated token
    pub fn refresh(&self) -> Result<Token> {
        let mut res = reqwest::get(&("https://auth.gog.com/token?client_id=46899977096215655&client_secret=9d85c43b1482497dbbce61f6e4aa173a433796eeae2ca8c5f6129f2dc4de46d9&grant_type=refresh_token&redirect_uri=https://embed.gog.com/on_login_success?origin=client&refresh_token=".to_string()+&self.refresh_token))?;
        Ok(serde_json::from_str(&res.text()?)?)
    }
    pub fn login(username: impl Into<String>, password: impl Into<String>) -> Result<Token> {
        let (username, password) = (username.into(), password.into());
        let garegex =
            Regex::new(r"var galaxyAccounts = new GalaxyAccounts\('(.+)','(.+)'\)").unwrap();
        let gsregex = Regex::new(r"(galaxy-login-s=.+;) expires").unwrap();
        let mut client = ReqwestSession::new(reqwest::ClientBuilder::new().build().unwrap());
        info!("Fetching GOG home page to get auth url");
        let mut result = client.get("https://gog.com").map_err(convert_rsession)?;
        let text = result
            .text()
            .expect("Couldn't get home page text")
            .to_owned()
            .to_string();
        if let Some(captures) = garegex.captures(&text) {
            let auth_url = captures[1].to_string();
            println!("Auth URl: {}", auth_url);
            info!("Got URL, requesting auth page");
            let mut aresult = client.get(&auth_url).map_err(convert_rsession)?;
            info!("Auth page request successful");
            let atext = aresult.text().expect("Couldn't get auth page text");
            println!("{}", atext);
            let document = Document::from(atext.as_str());
            info!("Checking for captchas");
            let gcaptcha = document.find(Class("g-recaptcha"));
            if gcaptcha.count() > 0 {
                error!("Captcha detected. Wait and try again.");
                Err(NotAvailable.into())
            } else {
                let mut login_id = document.find(Attr("id", "login__token"));
                if let Some(input) = login_id.next() {
                    info!("Got login ID");
                    let lid = input
                        .attr("value")
                        .expect("Login id field has no value.")
                        .to_string();
                    info!("Searching home page text with regex for url");
                    let mut form_parameters = std::collections::HashMap::new();
                    form_parameters.insert("login[username]", username);
                    form_parameters.insert("login[password]", password);
                    form_parameters.insert("login[login_flow]", "default".to_string());
                    form_parameters.insert("login[_token]", lid);
                    let check_url =
                        reqwest::Url::parse("https://login.gog.com/login_check").unwrap();
                    let mut request = client
                        .client
                        .post_request(&check_url)
                        .form(&form_parameters);
                    request = request
                        .add_cookies(client.store.get_request_cookies(&check_url).collect())
                        .header(reqwest::header::HOST, "login.gog.com");
                    println!("{:?}", request);
                    let mut login_response = request.send()?;
                    let login_text = login_response.text().expect("Couldn't fetch login text");
                    let login_doc = Document::from(login_text.as_str());
                    let mut two_step_search =
                        login_doc.find(Attr("id", "second_step_authentication__token"));
                    let url = login_response.url().clone();
                    if let Some(two_node) = two_step_search.next() {
                        info!("Two step authentication token needed.");
                        println!("Two step token required");
                        let two_token = two_node
                            .attr("value")
                            .expect("No two step token found")
                            .to_string();
                        Err(IncorrectCredentials.into())
                    } else {
                        println!("{:?}", client.store);
                        if url.as_str().contains("on_login_success") {
                            println!("{:?}", url);
                            let code = url
                                .query_pairs()
                                .filter(|(k, _v)| k == "code")
                                .map(|x| x.1)
                                .next()
                                .unwrap();
                            Token::from_login_code(code)
                        } else {
                            println!("{:?}", url);
                            println!("{:?}", login_response);
                            println!("LOGIN TEXT:{}", login_text);
                            error!("Login failed.");
                            Err(IncorrectCredentials.into())
                        }
                    }
                } else {
                    Err(MissingField("login id".to_string()).into())
                }
            }
        } else {
            Err(MissingField("auth url".to_string()).into())
        }
    }
}
