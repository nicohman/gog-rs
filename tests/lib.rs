use std::process::Command;
use std::env::var_os;
extern crate gog;
use gog::token::Token;
use gog::Gog;
#[test]
fn get_token() {
    let uri = "auth.gog.com/auth?client_id=46899977096215655&redirect_uri=https://embed.gog.com/on_login_success?origin=client&response_type=code&layout=client2";
    let token = Token::from_login_code(&var_os("GOG_KEY").unwrap().into_string().unwrap()).unwrap();
    println!("{:?}",token);
    let gog = Gog::new(token);
    println!("{:?}", gog.get_user_data().unwrap());
    let exp :Vec<String> = vec!["friendStatus".to_string(),"wishlistStatus".to_string(),"blockedStatus".to_string()];
    println!("{:?}", gog.get_pub_info(49171277422358, exp).unwrap());
    println!("{:?}", gog.get_games().unwrap());
    println!("{:?}", gog.get_game_details(1452598881).unwrap());
    println!("{:?}", gog.wishlist().unwrap());
    println!("{:?}",gog.add_wishlist(1096313866).unwrap());
    println!("{:?}:", gog.rm_wishlist(1096313866).unwrap());
    let exp_game : Vec<String> = vec!["downloads".to_string(),"description".to_string(),"screenshots".to_string(),"related_products".to_string()];
    println!("{:?}", gog.product(vec![1452598881,1096313866], exp_game).unwrap());
    println!("{:?}", gog.game_ratings().unwrap());
}

