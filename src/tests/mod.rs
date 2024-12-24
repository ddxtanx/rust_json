use crate::json::JSON;
use std::str::FromStr;
use std::time::Instant;

#[cfg(test)]
#[test]
fn test_arr() {
    let obj1 = JSON::String("hello".to_string());
    let obj2 = JSON::Number(42.0);
    let obj3 = JSON::Bool(true);
    let arr = JSON::Array(vec![obj1, obj2, obj3]);
    assert_eq!(arr.to_string(), "[\"hello\", 42, true]");
}

#[test]
fn test_arr_from_str() {
    let str = "[\"hello mister!\", 42, true]";
    let arr = JSON::from_str(str).unwrap();
    match arr.at(0) {
        Some(JSON::String(s)) => assert_eq!(s, "hello mister!"),
        _ => panic!("arr[0] is not a string"),
    }

    match arr.at(1) {
        Some(JSON::Number(n)) => assert_eq!(*n, 42.0),
        _ => panic!("arr[1] is not a number"),
    }
}

#[test]
fn test_obj_from_str() {
    let str = r#"{"name": "John", "age": 42, "is_student": false, "jobs": ["student", "teacher", "employee", {"type": "actor", "show": "phantom"}]}"#;
    let obj = JSON::from_str(str).unwrap();

    let name = obj.get("name").unwrap();
    match name {
        JSON::String(s) => assert_eq!(s, "John"),
        _ => panic!("name is not a string"),
    }

    let age = obj.get("age").unwrap();
    match age {
        JSON::Number(n) => assert_eq!(*n, 42.0),
        _ => panic!("age is not a number"),
    }

    let is_student = obj.get("is_student").unwrap();
    match is_student {
        JSON::Bool(b) => assert!(!(*b)),
        _ => panic!("is_student is not a boolean"),
    }

    let jobs = obj.get("jobs").unwrap();

    let job1 = jobs.at(0).unwrap();
    match job1 {
        JSON::String(s) => assert_eq!(s, "student"),
        _ => panic!("job1 is not a string"),
    }

    let job2 = jobs.at(1).unwrap();
    match job2 {
        JSON::String(s) => assert_eq!(s, "teacher"),
        _ => panic!("job2 is not a string"),
    }

    let job3 = jobs.at(2).unwrap();
    match job3 {
        JSON::String(s) => assert_eq!(s, "employee"),
        _ => panic!("job3 is not a string"),
    }

    let job4 = jobs.at(3).unwrap();

    match job4.get("type") {
        Some(JSON::String(s)) => assert_eq!(s, "actor"),
        _ => panic!("job4 type is not a string"),
    }

    match job4.get("show") {
        Some(JSON::String(s)) => assert_eq!(s, "phantom"),
        _ => panic!("job4 show is not a string"),
    }
}

#[test]
fn test_bad_parse() {
    let str = r#"{"name": "Jo\"hn", [1,2,3]: "asd"}"#;
    let obj = JSON::from_str(str);
    assert!(obj.is_err());
}

#[test]
fn test_big() {
    let file = std::fs::read_to_string("src/tests/users_100k.json").unwrap();
    let start = Instant::now();
    let obj = JSON::from_str(&file).expect("JSON should be valid");
    let elapsed = start.elapsed();
    println!("Parsed large users json in {:.2?}", elapsed);

    let vec = obj.as_array();
    assert!(vec.is_some());

    let vec = vec.unwrap();
    assert_eq!(vec.len(), 100_000);

    for i in 0..100_000 {
        let user = obj.at(i);
        assert!(user.is_some());
        let user = user.unwrap();
        match user {
            JSON::Object(_) => {
                let id = user.get("id").unwrap();
                match id {
                    JSON::Number(n) => assert_eq!(*n, i as f64),
                    _ => panic!("id is not a number"),
                }

                let name = user.get("name").unwrap();
                match name {
                    JSON::String(_) => (),
                    _ => panic!("name is not a string"),
                }

                let city = user.get("city").unwrap();
                match city {
                    JSON::String(_) => (),
                    _ => panic!("city is not a string"),
                }

                let age = user.get("age").unwrap();
                match age {
                    JSON::Number(_) => (),
                    _ => panic!("age is not a number"),
                }

                let friends = user.get("friends").unwrap();
                let friends = friends.as_array().unwrap();
                for friend in friends {
                    match friend {
                        JSON::Object(_) => {
                            let name = friend.get("name").unwrap();
                            match name {
                                JSON::String(_) => (),
                                _ => panic!("friend name is not a string"),
                            }

                            let hobbies = friend.get("hobbies").unwrap();
                            let hobbies = hobbies.as_array().unwrap();
                            for hobby in hobbies {
                                match hobby {
                                    JSON::String(_) => (),
                                    _ => panic!("hobby is not a string"),
                                }
                            }
                        }
                        _ => panic!("friend is not an object"),
                    }
                }
            }
            _ => panic!("user is not an object"),
        }
    }
    let should_fail = obj.at(100_000);
    assert!(should_fail.is_none());
}

#[test]
fn large_complex() {
    let file = std::fs::read_to_string("src/tests/large-complex.json").unwrap();
    let start = Instant::now();
    let json = JSON::from_str(&file).expect("JSON should be valid");
    let elapsed = start.elapsed();
    println!("Parsed large complex JSON in {:.2?}", elapsed);

    let len: usize = 11351;

    match &json {
        JSON::Array(arr) => assert_eq!(arr.len(), len),
        _ => {
            panic!("Obj is not array")
        }
    }

    for i in 0..len {
        let obj = json.at(i);
        assert!(obj.is_some());
        let obj = obj.unwrap();
        match &obj {
            JSON::Object(_) => (),
            _ => panic!("Expected object, didnt get"),
        }

        assert!(obj.get("id").is_some());
        assert!(obj.get("type").is_some());

        let actor = obj.get("actor");
        assert!(actor.is_some());
        let actor = actor.unwrap();

        match &actor {
            JSON::Object(_) => (),
            _ => panic!("Expected object, didnt get"),
        }

        assert!(actor.get("id").is_some());
        assert!(actor.get("login").is_some());
    }
}
