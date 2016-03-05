#![feature(plugin)]
#![plugin(confy(file="tests/Config.toml"))]

#[test]
fn read_toml() {
    assert_eq!(config!("key"), "value");
}
