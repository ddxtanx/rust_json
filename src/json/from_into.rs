use std::collections::HashMap;

use super::JSONValue;

impl From<f64> for JSONValue {
    fn from(value: f64) -> Self {
        JSONValue::Number(value)
    }
}

impl From<String> for JSONValue {
    fn from(value: String) -> Self {
        JSONValue::String(value)
    }
}

impl From<bool> for JSONValue {
    fn from(value: bool) -> Self {
        JSONValue::Bool(value)
    }
}

impl From<Vec<JSONValue>> for JSONValue {
    fn from(value: Vec<JSONValue>) -> Self {
        JSONValue::Array(value)
    }
}

impl From<HashMap<String, JSONValue>> for JSONValue {
    fn from(value: HashMap<String, JSONValue>) -> Self {
        JSONValue::Object(value)
    }
}

impl From<&str> for JSONValue {
    fn from(value: &str) -> Self {
        JSONValue::String(value.to_string())
    }
}
