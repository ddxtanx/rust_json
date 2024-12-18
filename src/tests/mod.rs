#[cfg(test)]
use super::*;

#[test]
fn test_arr() {
    let obj1 = JSONValue::String("hello".to_string());
    let obj2 = JSONValue::Number(42.0);
    let obj3 = JSONValue::Bool(true);
    let arr = JSONValue::Array(vec![obj1, obj2, obj3]);
    assert_eq!(arr.to_string(), "[\"hello\", 42, true]");
}

#[test]
fn test_arr_from_str() {
    let str = "[\"hello mister!\", 42, true]";
    let arr = JSONValue::from_str(str).unwrap();
    match arr.at(0) {
        Some(JSONValue::String(s)) => assert_eq!(s, "hello mister!"),
        _ => panic!("arr[0] is not a string"),
    }

    match arr.at(1) {
        Some(JSONValue::Number(n)) => assert_eq!(*n, 42.0),
        _ => panic!("arr[1] is not a number"),
    }
}

#[test]
fn test_obj_from_str() {
    let str = r#"{"name": "John", "age": 42, "is_student": false, "jobs": ["student", "teacher", "employee", {"type": "actor", "show": "phantom"}]}"#;
    let obj = JSONValue::from_str(str).unwrap();

    let name = obj.get("name").unwrap();
    match name {
        JSONValue::String(s) => assert_eq!(s, "John"),
        _ => panic!("name is not a string"),
    }

    let age = obj.get("age").unwrap();
    match age {
        JSONValue::Number(n) => assert_eq!(*n, 42.0),
        _ => panic!("age is not a number"),
    }

    let is_student = obj.get("is_student").unwrap();
    match is_student {
        JSONValue::Bool(b) => assert!(!(*b)),
        _ => panic!("is_student is not a boolean"),
    }

    let jobs = obj.get("jobs").unwrap();

    let job1 = jobs.at(0).unwrap();
    match job1 {
        JSONValue::String(s) => assert_eq!(s, "student"),
        _ => panic!("job1 is not a string"),
    }

    let job2 = jobs.at(1).unwrap();
    match job2 {
        JSONValue::String(s) => assert_eq!(s, "teacher"),
        _ => panic!("job2 is not a string"),
    }

    let job3 = jobs.at(2).unwrap();
    match job3 {
        JSONValue::String(s) => assert_eq!(s, "employee"),
        _ => panic!("job3 is not a string"),
    }

    let job4 = jobs.at(3).unwrap();

    match job4.get("type") {
        Some(JSONValue::String(s)) => assert_eq!(s, "actor"),
        _ => panic!("job4 type is not a string"),
    }

    match job4.get("show") {
        Some(JSONValue::String(s)) => assert_eq!(s, "phantom"),
        _ => panic!("job4 show is not a string"),
    }
}

#[test]
fn test_bad_parse() {
    let str = r#"{"name": "Jo\"hn", [1,2,3]: "asd"}"#;
    let obj = JSONValue::from_str(str);
    println!("{:?}", obj);
    assert!(obj.is_err());
}
