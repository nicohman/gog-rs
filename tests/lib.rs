use std::process::Command;
extern crate gog;
use gog::token::Token;
use gog::Gog;
#[test]
#[ignore]
fn get_token() {
    let uri = "auth.gog.com/auth?client_id=46899977096215655&redirect_uri=https://embed.gog.com/on_login_success?origin=client&response_type=code&layout=client2";
    let token = Token::from_login_code("***REMOVED***").unwrap();
    println!("{:?}",token);
    let gog = Gog::new(token);
    println!("{:?}", gog.get_user_data());
}

