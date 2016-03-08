#![feature(plugin)]
#![plugin(confy(file="tests/Config.toml"))]

#[test]
fn read_toml() {
    assert_eq!(config!("string"), "value");
    assert_eq!(config!("int"), 42);
    assert_eq!(config!("bool"), false);
    assert_eq!(config!("float"), 78.8);
    assert_eq!(config!("array"), vec![3, 4, 5]);
}

#[test]
fn default_value() {
    assert_eq!(config!("no-here/at all", 42), 42);
}
