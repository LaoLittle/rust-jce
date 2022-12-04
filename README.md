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

### Structs
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
    home: Home, // tag = 11
}

#[derive(JceStruct, PartialEq, Debug)]
struct Home {
    location: String, // tag = 0
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
    person.encode(&mut b).unwrap();
    println!("{:?}", &*b);
    let decode = Person::decode(&*b).unwrap();

    assert_eq!(person, decode);
}
```

### Fields
| Jce Type                  | Rust Type                  |
|---------------------------|----------------------------|
| BYTE                      | i8 / u8                    |
| SHORT                     | i16 / u16                  |
| INT                       | i32 / u32                  |
| LONG                      | i64 / u64                  |
| FLOAT                     | f32                        |
| DOUBLE                    | f64                        |
| SHORT_BYTES / LONG_BYTES  | Vec\<u8\> / Bytes / String |
| MAP                       | HashMap\<K, V\>            |
| LIST                      | Vec\<T\>                   |
| STRUCT_START + STRUCT_END | JceStruct                  |
| EMPTY                     | None::\<T\>                |
| SINGLE_LIST               | Vec\<u8\> / Bytes          |
