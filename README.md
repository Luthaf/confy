# Confy: compile-time configuration

Confy is a Rust compiler plugin for reading and using a TOML configuration file
at compile-time.

## Usage example

Add Confy to your `Cargo.toml`
```toml
[dependencies]
confy = "0.1.0"
```

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
