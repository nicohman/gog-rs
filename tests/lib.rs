use std::process::Command;
extern crate gog;
use gog::token::Token;
use gog::Gog;
#[test]
fn get_token() {
    let uri = "auth.gog.com/auth?client_id=46899977096215655&redirect_uri=https://embed.gog.com/on_login_success?origin=client&response_type=code&layout=client2";
    let token = Token::from_login_code("***REMOVED***").unwrap();
    println!("{:?}",token);
    let gog = Gog::new(token);
    println!("{:?}", gog.get_user_data());
    let exp :Vec<String> = vec!["friendStatus".to_string(),"wishlistStatus".to_string(),"blockedStatus".to_string()];
    println!("{:?}", gog.get_pub_info(49171277422358, exp));
    println!("{:?}", gog.get_games());
    println!("{:?}", gog.get_game_details(1452598881));
    println!("{:?}", gog.wishlist());
    println!("{:?}",gog.add_wishlist(1096313866));
    println!("{:?}", gog.rm_wishlist(1096313866));
}

