#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate reqwest;
pub mod token;
pub mod gog;
use gog::*;
use token::Token;
use serde_json::value::Value;
use reqwest::Client;
pub struct Gog {
    pub token:Token,
    pub client: Client
}
impl Gog {
    pub fn from_login_code(code: &str) -> Gog {
        Gog::from_token(Token::from_login_code(code).unwrap())
    }
    pub fn new(token: Token) -> Gog {
        Gog::from_token(token)
    }
    fn from_token(token: Token) -> Gog {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", ("Bearer ".to_string()+&token.access_token).parse().unwrap());
        let mut client = Client::builder();
        client = client.default_headers(headers);
        return Gog {
            token:token,
            client:client.build().unwrap()
        };
    }
    pub fn get_user_data(&self) -> UserData {
         let udata : UserData = serde_json::from_str(&self.client.get("https://embed.gog.com/userData.json").send().unwrap().text().unwrap()).unwrap();
         udata
    }
}
