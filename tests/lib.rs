use std::process::Command;
extern crate gog;
use gog::Token;
#[test]
#[ignore]
fn get_token() {
    let uri = "auth.gog.com/auth?client_id=46899977096215655&redirect_uri=https://embed.gog.com/on_login_success?origin=client&response_type=code&layout=client2";
    let token = Token::from_login_code("***REMOVED***".to_string());
    println!("{:?}",token);
}
