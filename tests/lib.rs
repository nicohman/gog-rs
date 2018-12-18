use std::process::Command;
use std::env::var_os;
use std::fs::File;
extern crate gog;
use gog::token::Token;
use gog::Gog;
use gog::gog::FilterParam::*;
use gog::gog::*;
use std::io::Read;
fn get_gog() -> Gog {
    let path = var_os("GOG_TOKEN_PATH").unwrap().into_string().unwrap();
    let mut token_json = String::new();
    File::open(path).unwrap().read_to_string(&mut token_json).unwrap();
    let uri = "auth.gog.com/auth?client_id=46899977096215655&redirect_uri=https://embed.gog.com/on_login_success?origin=client&response_type=code&layout=client2";
    let mut token = Token::from_response(&token_json).unwrap();
    token = token.refresh().unwrap();
    let mut gog = Gog::new(token);
    gog
}
#[test]
fn get_games() {
    let gog = get_gog();
    gog.get_games().unwrap();
}
#[test]
fn get_pub_info() {
    let gog = get_gog();
    let exp :Vec<String> = vec!["friendStatus".to_string(),"wishlistStatus".to_string(),"blockedStatus".to_string()];
    gog.get_pub_info(49171277422358, exp).unwrap();
}
#[test]
fn get_game_details()
{
    let gog = get_gog();
    gog.get_game_details(1452598881).unwrap();
}

#[test]
fn wishlist() {
    let gog = get_gog();
    gog.wishlist().unwrap();
}
#[test]
fn add_wishlist() {
    let gog = get_gog();
    gog.add_wishlist(1096313866).unwrap();
}
#[test]
fn rm_wishlist() {
    let gog = get_gog();
    gog.rm_wishlist(1096313866).unwrap();
}
#[test]
fn product() {
    let exp_game : Vec<String> = vec!["downloads".to_string(),"description".to_string(),"screenshots".to_string(),"related_products".to_string()];
    let gog = get_gog();
    gog.product(vec![1452598881,1096313866], exp_game).unwrap();
}
#[test]
fn game_ratings() {
    let gog = get_gog();
    gog.game_ratings().unwrap();
}
#[test]
fn filtered() {
    let gog = get_gog();
    gog.get_filtered_products(FilterParams::from_one(MediaType(1))).unwrap();
}
