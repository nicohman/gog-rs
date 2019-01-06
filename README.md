# gog-rs [![](https://img.shields.io/crates/v/gog.svg?style=flat-square)](https://crates.io/crates/gog) [![builds.sr.ht status](https://builds.sr.ht/~nicohman/gogapi-rs.svg)](https://builds.sr.ht/~nicohman/gogapi-rs?)

gog-rs is a rust library for talking to [GOG's unofficial REST API](https://gogapidocs.readthedocs.io/en/latest/index.html). Many thanks to Yepoleb for the hard work documenting how GOG's API works. This library is written mostly to support [wyvern](https://git.sr.ht/~nicohman/wyvern), but if you want any other endpoints/methods to be implemented, let me know and I'll be happy to add them! This is a mirror for the [sr.ht repository](https://git.sr.ht/~nicohman/gogapi-rs)

## Getting started

- [Documentation](https://docs.rs/gog)

- You can also find soome good examples in both the [tests](https://git.sr.ht/%7Enicohman/gogapi-rs/tree/master/tests/lib.rs) and [wyvern](https://git.sr.ht/~nicohman/wyvern)

### Example

```
// Gets a list of the ids of all games the user owns
let token = gog::token::Token::from_login_code(/*This code is from the GOG OAuth login page*/).unwrap();
let gog = gog::Gog::new(token);
println!("{:?}", gog.get_games().unwrap())
// [6, 1146738698, 1207658679, 1207658691, 1207658695...]
```
