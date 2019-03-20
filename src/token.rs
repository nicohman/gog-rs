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
        let text = res.text()?;
        Token::from_response(text)
    }
    pub fn from_home_code(code: impl Into<String>) -> Result<Token> {
        let url = format!("https://auth.gog.com/token?client_id=46899977096215655&client_secret=9d85c43b1482497dbbce61f6e4aa173a433796eeae2ca8c5f6129f2dc4de46d9&grant_type=authorization_code&redirect_uri=https%3A%2F%2Fwww.gog.com%2Fon_login_success&layout=client2&code={}", code.into());
        let mut res = reqwest::get(&url)?;
        let text = res.text()?;
        Token::from_response(text)
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
    pub fn login<F>(
        username: impl Into<String>,
        password: impl Into<String>,
        two_step_token_fn: F,
    ) -> Result<Token>
    where
        F: Fn() -> String,
    {
        let (username, password) = (username.into(), password.into());
        let garegex =
            Regex::new(r"var galaxyAccounts = new GalaxyAccounts\('(.+)','(.+)'\)").unwrap();
        let mut client = ReqwestSession::new(
            reqwest::ClientBuilder::new()
                .redirect(reqwest::RedirectPolicy::none())
                .build()
                .unwrap(),
        );
        let mut normal_client = ReqwestSession::new(reqwest::ClientBuilder::new().build().unwrap());
        info!("Fetching GOG home page to get auth url");
        let mut result = normal_client
            .get("https://gog.com")
            .map_err(convert_rsession)?;
        let text = result
            .text()
            .expect("Couldn't get home page text")
            .to_owned()
            .to_string();
        if let Some(captures) = garegex.captures(&text) {
            let auth_url = captures[1].to_string();
            info!("Got Auth URL as {}, requesting auth page", auth_url);
            let mut aresult = client.get(&auth_url).map_err(convert_rsession)?;
            while aresult.status().is_redirection() {
                let mut next_url = aresult
                    .headers()
                    .get("Location")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                aresult = client
                    .get(reqwest::Url::parse(&next_url).unwrap())
                    .map_err(convert_rsession)?
            }
            info!("Auth page request successful");
            let atext = aresult.text().expect("Couldn't get auth page text");
            let document = Document::from(atext.as_str());
            info!("Checking for captchas");
            let gcaptcha = document.find(Attr("defer", ""));
            for poss in gcaptcha {
                if poss.html().contains("recaptcha") {
                    error!("Captcha detected. Wait and try again.");
                    println!("Captcha");
                    return Err(NotAvailable.into());
                }
            }
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
                form_parameters.insert("login[login]", String::default());
                form_parameters.insert("login[login_flow]", "default".to_string());
                form_parameters.insert("login[_token]", lid);
                let check_url = reqwest::Url::parse("https://login.gog.com/login_check").unwrap();
                let mut request = client
                    .client
                    .post_request(&check_url)
                    .form(&form_parameters);
                let mut cookies_processed: Vec<cookie::Cookie> = client
                    .store
                    .get_request_cookies(&check_url)
                    .cloned()
                    .collect();
                request = request.add_cookies(cookies_processed.iter().collect());
                let mut login_response = request.send()?;
                while login_response.status().is_redirection() {
                    let mut next_url = login_response
                        .headers()
                        .get("Location")
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    login_response = client
                        .client
                        .get_request(&reqwest::Url::parse(&next_url).unwrap())
                        .add_cookies(cookies_processed.iter().collect())
                        .send()?;
                }
                let login_text = login_response.text().expect("Couldn't fetch login text");
                let login_doc = Document::from(login_text.as_str());
                let mut two_step_search =
                    login_doc.find(Attr("id", "second_step_authentication__token"));
                let url = login_response.url().clone();
                if let Some(two_node) = two_step_search.next() {
                    error!("Two step authentication token needed.");
                    println!("Two step token required. This is not implemented yet.");
                    let two_token_secret = two_node
                        .attr("value")
                        .expect("No two step token found")
                        .to_string();
                    let two_token = two_step_token_fn();
                    if two_token.len() < 4 {
                        return Err(MissingField("Token too short".to_string()).into());
                    }
                    let mut token_iter = two_token.chars().map(|x| x.to_string());
                    let mut token_parameters = std::collections::HashMap::new();
                    token_parameters.insert(
                        "second_step_authentication[token][letter_1]",
                        token_iter.next().unwrap(),
                    );
                    token_parameters.insert(
                        "second_step_authentication[token][letter_2]",
                        token_iter.next().unwrap(),
                    );
                    token_parameters.insert(
                        "second_step_authentication[token][letter_3]",
                        token_iter.next().unwrap(),
                    );
                    token_parameters.insert(
                        "second_step_authentication[token][letter_4]",
                        token_iter.next().unwrap(),
                    );
                    token_parameters.insert("second_step_authentication[send]", String::default());
                    token_parameters.insert("second_step_authentication[_token]", two_token_secret);
                    let mut login_response = client
                        .client
                        .post_request(&url)
                        .add_cookies(cookies_processed.iter().collect())
                        .send()?;
                    while login_response.status().is_redirection() {
                        let mut next_url = login_response
                            .headers()
                            .get("Location")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();
                        login_response = client
                            .client
                            .get_request(&reqwest::Url::parse(&next_url).unwrap())
                            .add_cookies(cookies_processed.iter().collect())
                            .send()?;
                    }
                    if url.as_str().contains("on_login_success") {
                        let code = url
                            .query_pairs()
                            .filter(|(k, _v)| k == "code")
                            .map(|x| x.1)
                            .next()
                            .unwrap();
                        Token::from_home_code(code)
                    } else {
                        error!("Login failed. Incorrect Two factor Token");
                        Err(IncorrectCredentials.into())
                    }
                } else {
                    if url.as_str().contains("on_login_success") {
                        let code = url
                            .query_pairs()
                            .filter(|(k, _v)| k == "code")
                            .map(|x| x.1)
                            .next()
                            .unwrap();
                        Token::from_home_code(code)
                    } else {
                        error!("Login failed. Incorrect credentials");
                        Err(IncorrectCredentials.into())
                    }
                }
            } else {
                Err(MissingField("login id".to_string()).into())
            }
        } else {
            Err(MissingField("auth url".to_string()).into())
        }
    }
}
