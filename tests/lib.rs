use std::env::var_os;
use std::fs::File;
extern crate gog;
use gog::gog::FilterParam::*;
use gog::gog::OS::*;
use gog::gog::*;
use gog::token::Token;
use gog::*;
use std::io::Read;

// Some tests need an owned game.
// "Beneath a Steel Sky" is free from GOG, so a good test case.
const OWNED_GAME_ID: i64 = 1207658695;

fn get_gog() -> Gog {
    let path = var_os("GOG_TOKEN_PATH").unwrap().into_string().unwrap();
    let mut token_json = String::new();
    File::open(path)
        .unwrap()
        .read_to_string(&mut token_json)
        .unwrap();
    let mut token = Token::from_response(token_json.as_str()).unwrap();
    token = token.refresh().unwrap();
    Gog::new(token)
}

#[test]
fn get_games() {
    let gog = get_gog();
    gog.get_games().unwrap();
}

#[test]
fn get_pub_info() {
    let gog = get_gog();
    let exp: Vec<String> = vec![
        "friendStatus".to_string(),
        "wishlistStatus".to_string(),
        "blockedStatus".to_string(),
    ];
    gog.get_pub_info(49171277422358, exp).unwrap();
}

#[test]
fn get_game_details() {
    let gog = get_gog();
    gog.get_game_details(OWNED_GAME_ID).unwrap();
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
    let exp_game: Vec<String> = vec![
        "downloads".to_string(),
        "description".to_string(),
        "screenshots".to_string(),
        "related_products".to_string(),
    ];
    let gog = get_gog();
    gog.product(vec![1452598881, 1096313866], exp_game).unwrap();
}

#[test]
fn game_ratings() {
    let gog = get_gog();
    gog.game_ratings().unwrap();
}

#[test]
fn filtered() {
    let gog = get_gog();
    gog.get_filtered_products(FilterParams::from_one(MediaType(1)))
        .unwrap();
}

#[test]
fn filtered_all() {
    let gog = get_gog();
    gog.get_all_filtered_products(FilterParams::from_one(MediaType(1)))
        .unwrap();
}

#[test]
fn filtered_os() {
    let gog = get_gog();
    gog.get_filtered_products(FilterParams::from_one(OS(Linux)))
        .unwrap();
}

#[test]
fn download() {
    let gog = get_gog();
    let details = gog.get_game_details(OWNED_GAME_ID).unwrap();
    gog.download_game(details.downloads.linux.unwrap());
}

#[test]
fn filtered_search() {
    let gog = get_gog();
    gog.get_filtered_products(FilterParams::from_one(Search("Not A Hero".to_string())))
        .unwrap();
}

#[test]
fn filtered_unowned() {
    let gog = get_gog();
    gog.get_products(FilterParams::from_one(Search("Stellaris".to_string())))
        .unwrap();
}

#[test]
fn connect_status() {
    let gog = get_gog();
    let uid = gog.get_user_data().unwrap().user_id;
    let status = gog.connect_status(uid.parse().unwrap());
    if status.is_err() {
        let err = status.err().unwrap();
        match err.kind() {
            ErrorKind::NotAvailable => {}
            _ => panic!("{:?}", err),
        }
    }
}

#[test]
fn friends() {
    let gog = get_gog();
    gog.friends().unwrap();
}

#[test]
fn extract_data() {
    let gog = get_gog();
    let details = gog.get_game_details(OWNED_GAME_ID).unwrap();
    assert!(gog.extract_data(details.downloads.linux.unwrap()).is_ok())
}

#[test]
#[ignore]
fn extract() {
    use gog::extract::*;
    let mut in_file = File::open("beneath_a_steel_sky_en_gog_2_20150.sh").unwrap();
    extract(
        &mut in_file,
        ".",
        ToExtract {
            unpacker: true,
            data: true,
            mojosetup: true,
        },
    )
    .unwrap();
}
