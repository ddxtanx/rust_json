mod tests;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    str::FromStr,
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

#[derive(Debug)]
pub enum JSONError {
    UnexpectedCharacter(char, usize, usize),
    UnexpectedEndOfInput,
    ParseError(&'static str),
}

#[derive(Debug)]
enum ParsingHelper {
    ObjStart,
    KeyStart,
    KeyEnd,
    ValueStart,
    ValueEnd,
    ObjEnd,
    ArrayStart,
    ElementStart,
    ElementEnd,
    ArrayEnd,
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

fn parse_partial(tokens: &[ParsingHelper]) -> Result<(JSONValue, &[ParsingHelper]), JSONError> {
    use ParsingHelper::*;
    if tokens.is_empty() {
        return Err(JSONError::UnexpectedEndOfInput);
    }

    let first = tokens.first().unwrap();
    match first {
        String(s) => Ok((JSONValue::String(s.clone()), &tokens[1..])),
        Number(n) => Ok((JSONValue::Number(*n), &tokens[1..])),
        Bool(b) => Ok((JSONValue::Bool(*b), &tokens[1..])),
        Null => Ok((JSONValue::Null, &tokens[1..])),
        ObjStart => {
            let mut obj = HashMap::new();
            let mut slice: &[ParsingHelper] = &tokens[1..];
            loop {
                if slice.is_empty() {
                    return Err(JSONError::UnexpectedEndOfInput);
                };

                match slice[0] {
                    ObjEnd => {
                        break;
                    }
                    KeyStart => (),
                    _ => return Err(JSONError::ParseError("Expected key start")),
                }

                let key = match &slice[1] {
                    String(s) => s.clone(),
                    _ => return Err(JSONError::ParseError("Expected string key")),
                };

                match slice[2] {
                    KeyEnd => (),
                    _ => return Err(JSONError::ParseError("Expected key end")),
                }

                match slice[3] {
                    ValueStart => (),
                    _ => return Err(JSONError::ParseError("Expected value start")),
                }

                let (value, new_slice) = parse_partial(&slice[4..])?;
                obj.insert(key, value);
                slice = &new_slice[1..];
            }
            Ok((JSONValue::Object(obj), &slice[1..]))
        }
        ArrayStart => {
            let mut arr = Vec::new();
            let mut slice: &[ParsingHelper] = &tokens[1..];
            loop {
                if slice.is_empty() {
                    return Err(JSONError::UnexpectedEndOfInput);
                };

                match slice[0] {
                    ArrayEnd => {
                        break;
                    }
                    ElementStart => (),
                    _ => return Err(JSONError::ParseError("Expected element start")),
                }

                let (value, new_slice) = parse_partial(&slice[1..])?;
                arr.push(value);
                slice = &new_slice[1..];
            }
            Ok((JSONValue::Array(arr), &slice[1..]))
        }
        _ => Err(JSONError::ParseError("Unexpected token in partial parse")),
    }
}

impl FromStr for JSONValue {
    type Err = JSONError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut stack: Vec<ParsingHelper> = Vec::new();
        let mut tokens: Vec<String> = Vec::new();
        let control_chars = ['{', '}', '[', ']', ':', ','];
        let mut temp = String::new();

        let mut escaped = false;
        let mut in_string = false;

        for c in s.chars() {
            if !in_string && c.is_whitespace() {
                continue;
            }

            if c == '\\' {
                if !in_string {
                    return Err(JSONError::ParseError("Unexpected escape character"));
                }
                if escaped {
                    escaped = false;
                    temp.push(c);
                } else {
                    escaped = true;
                    continue;
                }
            }

            if c == '"' {
                temp.push(c);
                if !in_string {
                    in_string = true;
                } else {
                    in_string = false;
                    tokens.push(temp.clone());
                    temp.clear();
                }
                continue;
            }

            if in_string {
                temp.push(c);
                continue;
            }

            if control_chars.contains(&c) {
                if !temp.is_empty() {
                    tokens.push(temp.clone());
                    temp.clear();
                }
                tokens.push(c.to_string());
            } else {
                temp.push(c);
            }
        }

        let mut current_scope = Vec::new();
        for token in tokens {
            match token.as_str() {
                "{" => {
                    stack.push(ParsingHelper::ObjStart);
                    current_scope.push(ParsingHelper::ObjStart);
                    stack.push(ParsingHelper::KeyStart);
                }
                ":" => {
                    let scope = current_scope.last();
                    match scope {
                        Some(ParsingHelper::ObjStart) => {
                            stack.push(ParsingHelper::KeyEnd);
                            stack.push(ParsingHelper::ValueStart);
                        }
                        _ => {
                            return Err(JSONError::ParseError("Unexpected colon"));
                        }
                    }
                }
                "}" => {
                    let scope = current_scope.pop();
                    match scope {
                        Some(ParsingHelper::ObjStart) => {
                            stack.push(ParsingHelper::ValueEnd);
                            stack.push(ParsingHelper::ObjEnd);
                        }
                        _ => {
                            return Err(JSONError::ParseError("Unexpected end of object"));
                        }
                    }
                }
                "[" => {
                    stack.push(ParsingHelper::ArrayStart);
                    current_scope.push(ParsingHelper::ArrayStart);
                    stack.push(ParsingHelper::ElementStart);
                }
                "," => {
                    let scope = current_scope.last();
                    match scope {
                        Some(ParsingHelper::ObjStart) => {
                            stack.push(ParsingHelper::ValueEnd);
                            stack.push(ParsingHelper::KeyStart);
                        }
                        Some(ParsingHelper::ArrayStart) => {
                            stack.push(ParsingHelper::ElementEnd);
                            stack.push(ParsingHelper::ElementStart);
                        }
                        _ => {
                            return Err(JSONError::ParseError("Unexpected comma"));
                        }
                    }
                }
                "]" => {
                    let scope = current_scope.pop();
                    match scope {
                        Some(ParsingHelper::ArrayStart) => {
                            stack.push(ParsingHelper::ElementEnd);
                            stack.push(ParsingHelper::ArrayEnd);
                        }
                        _ => {
                            return Err(JSONError::ParseError("Unexpected end of array"));
                        }
                    }
                }
                "true" => {
                    stack.push(ParsingHelper::Bool(true));
                }
                "false" => {
                    stack.push(ParsingHelper::Bool(false));
                }
                "null" => {
                    stack.push(ParsingHelper::Null);
                }
                _ => {
                    if let Ok(num) = token.parse::<f64>() {
                        stack.push(ParsingHelper::Number(num));
                    } else {
                        let sub = &token[1..token.len() - 1];
                        stack.push(ParsingHelper::String(sub.to_string()));
                    }
                }
            }
        }

        let (value, remaining) = parse_partial(&stack)?;
        if !remaining.is_empty() {
            return Err(JSONError::ParseError("Unexpected tokens at end of input"));
        }
        Ok(value)
    }
}

pub struct JSON {
    fields: HashMap<String, JSONValue>,
}

impl Display for JSON {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl FromStr for JSON {
    type Err = JSONError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = JSONValue::from_str(s)?;
        match value {
            JSONValue::Object(obj) => {
                let mut json = JSON::new();
                for (k, v) in obj {
                    json.add_field(&k, v);
                }
                Ok(json)
            }
            _ => Err(JSONError::ParseError("Expected object")),
        }
    }
}
