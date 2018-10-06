#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde;
extern crate reqwest;
pub mod token;
pub mod gog;
use gog::*;
use domains::*;
use ErrorType::*;
use token::Token;
use serde_json::value::{Map, Value};
use reqwest::{Client, Method, Response};
use std::result::Result;
use serde::de::DeserializeOwned;
const GET : Method = Method::GET;
const POST : Method = Method::POST;
macro_rules! map_p {
    ($($js: tt)+) => {
        Some(json!($($js)+).as_object().unwrap().clone())
    }
}
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
    fn rget(&self, domain: &str, path: &str , params:Option<Map<String, Value>>) -> Result<Response, reqwest::Error> {
        let mut url = domain.to_string()+path;
        if params.is_some() {
            let params = params.unwrap();
            if params.len() > 0 {
                url = url + "?";
                for (k, v) in params.iter() {
                    url = url + k+"="+&v.to_string() + "&";
                }
                url.pop();
            }
        }
        let res =  self.client.get(&url).send();
        return res;
    }
    fn fget <T> (&self, domain:&str, path:&str, params:Option<Map<String, Value>>) -> Result<T, Error> where T: DeserializeOwned, {
        let res = self.rget(domain, path, params);
        if res.is_err() {
            return Err(Error {
                etype: Req,
                error:Some(res.err().unwrap()),
                msg:None
            });
        } else {
            let st = res.unwrap().text().unwrap();
            let try : Result<T, serde_json::Error> = serde_json::from_str(&st);
            if try.is_ok() {
                return Ok(try.unwrap());
            } else {
                return Err(Error {
                    etype: Gog,
                    msg: Some(st),
                    error:None
                });
            }
        }
    }
    pub fn get_user_data(&self) -> Result<UserData, Error> {
        self.fget(EMBD, "/userData.json", None)
    }
    pub fn get_pub_info(&self, uid: i64, expand:Vec<String>) -> Result<PubInfo, Error> {
        let r : Result<PubInfo, Error> = self.fget(EMBD, &("/users/info/".to_string()+&uid.to_string()), map_p!({
            "expand":expand.iter().try_fold("".to_string(), fold_mult).unwrap()
        }));
        r
    }
}
fn fold_mult(acc: String, now: &String) -> Result<String, Error> {
    return Ok(acc +","+now);
}
