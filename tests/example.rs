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

#[test]
fn person() {
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
