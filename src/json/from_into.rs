use std::collections::HashMap;

use super::JSON;

impl From<f64> for JSON {
    fn from(value: f64) -> Self {
        JSON::Number(value)
    }
}

impl From<String> for JSON {
    fn from(value: String) -> Self {
        JSON::String(value)
    }
}

impl From<&String> for JSON {
    fn from(value: &String) -> Self {
        JSON::String(value.clone())
    }
}

impl From<bool> for JSON {
    fn from(value: bool) -> Self {
        JSON::Bool(value)
    }
}

impl From<&str> for JSON {
    fn from(value: &str) -> Self {
        JSON::String(value.to_string())
    }
}

impl<U, V> From<HashMap<U, V>> for JSON
where
    U: Into<String> + Eq + std::hash::Hash,
    V: Into<JSON>,
{
    fn from(value: HashMap<U, V>) -> Self {
        let mut map = HashMap::new();
        for (key, val) in value {
            map.insert(key.into(), val.into());
        }
        JSON::Object(map)
    }
}

impl<U> From<Vec<U>> for JSON
where
    U: Into<JSON>,
{
    fn from(value: Vec<U>) -> Self {
        JSON::Array(value.into_iter().map(|v| v.into()).collect())
    }
}
