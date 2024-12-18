use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result},
};

#[derive(Clone, Debug, PartialEq)]
pub enum JSONValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JSONValue>),
    Object(HashMap<String, JSONValue>),
}

impl JSONValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JSONValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            JSONValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            JSONValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JSONValue>> {
        match self {
            JSONValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JSONValue>> {
        match self {
            JSONValue::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JSONValue> {
        match self {
            JSONValue::Object(o) => o.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JSONValue> {
        match self {
            JSONValue::Object(o) => o.get_mut(key),
            _ => None,
        }
    }

    pub fn at(&self, index: usize) -> Option<&JSONValue> {
        match self {
            JSONValue::Array(a) => a.get(index),
            _ => None,
        }
    }

    pub fn at_mut(&mut self, index: usize) -> Option<&mut JSONValue> {
        match self {
            JSONValue::Array(a) => a.get_mut(index),
            _ => None,
        }
    }
}

impl Display for JSONValue {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            JSONValue::Null => write!(f, "null"),
            JSONValue::Bool(b) => write!(f, "{}", b),
            JSONValue::Number(n) => write!(f, "{}", n),
            JSONValue::String(s) => write!(f, "\"{}\"", s),
            JSONValue::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            JSONValue::Object(o) => {
                write!(f, "{{")?;
                for (i, (k, v)) in o.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}
pub struct JSON {
    fields: HashMap<String, JSONValue>,
}

impl Display for JSON {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", JSONValue::Object(self.fields.clone()))
    }
}

impl JSON {
    pub fn new() -> Self {
        JSON {
            fields: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, key: &str, value: JSONValue) {
        self.fields.insert(key.to_string(), value);
    }
}

impl Default for JSON {
    fn default() -> Self {
        Self::new()
    }
}
