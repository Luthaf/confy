# Confy: compile-time configuration

Confy was a Rust compiler plugin for reading and using a TOML configuration file
at compile-time.

**EDIT**: this is an abandonned experiment, the confy name on crates.io have since be transfered to https://github.com/rust-cli/confy.

## Usage example

And then you can use it like this:
```rust
#![feature(plugin)]
#![plugin(confy(file="Config.toml"))]

fn main() {
    assert_eq!(config!("string"), "value");
    assert_eq!(config!("int"), 42);
    assert_eq!(config!("bool"), false);
    assert_eq!(config!("float"), 78.8);
    assert_eq!(config!("array"), vec![3, 4, 5]);

    // You can specify a default, just in case
    assert_eq!(config!("key not found", "foo"), "foo");
}
```

You can retrieve all TOML types except for table: string, integers, floats, and
boolean values.

## License

Confy is licensed under either of Apache License Version 2.0 or MIT license at
your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
