pub mod from_into;

use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum JSON {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JSON>),
    Object(HashMap<String, JSON>),
}

pub enum JSONError {
    InsertError,
}

impl JSON {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JSON::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            JSON::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            JSON::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JSON>> {
        match self {
            JSON::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JSON>> {
        match self {
            JSON::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JSON> {
        match self {
            JSON::Object(o) => o.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JSON> {
        match self {
            JSON::Object(o) => o.get_mut(key),
            _ => None,
        }
    }

    pub fn at(&self, index: usize) -> Option<&JSON> {
        match self {
            JSON::Array(a) => a.get(index),
            _ => None,
        }
    }

    pub fn at_mut(&mut self, index: usize) -> Option<&mut JSON> {
        match self {
            JSON::Array(a) => a.get_mut(index),
            _ => None,
        }
    }

    pub fn insert(&mut self, key: String, value: JSON) -> Result<(), JSONError> {
        match self {
            JSON::Object(o) => {
                o.insert(key, value);
                Ok(())
            }
            _ => Err(JSONError::InsertError),
        }
    }

    pub fn push(&mut self, value: JSON) -> Result<(), JSONError> {
        match self {
            JSON::Array(a) => {
                a.push(value);
                Ok(())
            }
            _ => Err(JSONError::InsertError),
        }
    }
}

impl Display for JSON {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            JSON::Null => write!(f, "null"),
            JSON::Bool(b) => write!(f, "{}", b),
            JSON::Number(n) => write!(f, "{}", n),
            JSON::String(s) => write!(f, "\"{}\"", s),
            JSON::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            JSON::Object(o) => {
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
