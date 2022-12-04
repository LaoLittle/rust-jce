use jce::JceStruct;
use std::collections::HashMap;
use bytes::BytesMut;

#[test]
fn map() {
    #[derive(JceStruct, Debug)]
    struct M {
        map: HashMap<String, i16>,
    }

    println!(
        "{:?}",
        M::decode([8, 0, 2, 6, 2, 49, 50, 17, 1, 66, 6, 2, 49, 52, 17, 1, 66].as_ref()).unwrap()
    );

    let mut enc = BytesMut::new();
    M {
        map: {
            let mut map = HashMap::new();
            map.insert("123".into(), 114);
            map.insert("456".into(), 514);
            map
        }
    }.encode(&mut enc);

    println!("{:?}", &*enc);
}
