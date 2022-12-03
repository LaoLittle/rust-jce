use jce::JceStruct;

#[test]
fn option() {
    #[derive(JceStruct, Debug)]
    struct Optional {
        // tag = 0
        a: Option<u8>,
        // tag = 1
        b: Option<Vec<u8>>,
    }

    let op = Optional::decode([12].as_ref()).unwrap();

    assert!(op.a.is_none());
}
