# Rust-Jce

[crates-io]: https://crates.io/crates/jce
[crates-io-shields]: https://img.shields.io/crates/v/jce.svg
[docs-rs]: https://docs.rs/jce
[docs-rs-shields]: https://img.shields.io/badge/docs.rs-rustdoc-green.svg
[license]: https://github.com/LaoLittle/rust-jce/blob/master/LICENSE.md
[license-shields]: https://img.shields.io/crates/l/jce.svg

`jce` is a Jce encoding/decoding implementation for the
Rust programing language.

Why `jce`?

- Written in pure rust
- Easy-to-use

use a simple derive `JceStruct`

let's define a struct first.
```rust
use jce::JceStruct;
#[derive(JceStruct, PartialEq, Debug)]
struct Person {
    name: String, // tag = 0
    age: u8, // tag = 1
    #[jce(tag = "5")]
    male: bool, // tag = 5
    phone: u64, // tag = 6
    #[jce(tag = "11")]
    home: Home,
}

#[derive(JceStruct, PartialEq, Debug)]
struct Home {
    location: String,
}

fn main() {
    let person = Person {
        name: "Jack".into(),
        age: 12,
        male: true,
        phone: 1145141919810,
        home: Home {
            location: "下北泽".into()
        }
    };

    let mut b = vec![0u8; 0];
    person.encode(&mut b);
    println!("{:?}", &*b);
    let decode = Person::decode(&*b).unwrap();

    assert_eq!(person, decode);
}
```
