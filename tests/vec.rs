use bytes::Bytes;
use jce::JceStruct;

#[test]
fn vec() {
    #[derive(JceStruct, Debug)]
    struct V {
        a: Vec<i16>,
        d: Bytes,
    }

    let v = V {
        a: vec![114, 514],
        d: Bytes::from_static(&[114, 51, 4]),
    };

    let mut bytes = Vec::new();
    v.encode(&mut bytes);

    println!("{:?}", bytes);
    println!("{:?}", V::decode([9, 2, 0, 0, 0, 2, 1, 0, 114, 1, 2, 2, 16, 3, 114, 51, 4].as_ref()));
}
