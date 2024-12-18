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

impl From<bool> for JSON {
    fn from(value: bool) -> Self {
        JSON::Bool(value)
    }
}

impl From<Vec<JSON>> for JSON {
    fn from(value: Vec<JSON>) -> Self {
        JSON::Array(value)
    }
}

impl From<HashMap<String, JSON>> for JSON {
    fn from(value: HashMap<String, JSON>) -> Self {
        JSON::Object(value)
    }
}

impl From<&str> for JSON {
    fn from(value: &str) -> Self {
        JSON::String(value.to_string())
    }
}
