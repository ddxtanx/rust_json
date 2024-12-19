use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc, str::FromStr};

use crate::json::JSON;

#[derive(Debug)]
pub enum JSONError {
    UnexpectedCharacter(char, usize, usize),
    UnexpectedEndOfInput,
    ParseError(&'static str),
}

impl Display for JSONError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            JSONError::UnexpectedCharacter(c, l, p) => {
                write!(
                    f,
                    "Unexpected character '{}' at line {} position {}",
                    c, l, p
                )
            }
            JSONError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            JSONError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
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

fn parse_partial(tokens: &[ParsingHelper]) -> Result<(JSON, &[ParsingHelper]), JSONError> {
    use ParsingHelper::*;
    if tokens.is_empty() {
        return Err(JSONError::UnexpectedEndOfInput);
    }

    let first = tokens.first().unwrap();
    match first {
        String(s) => Ok((JSON::String(s.clone()), &tokens[1..])),
        Number(n) => Ok((JSON::Number(*n), &tokens[1..])),
        Bool(b) => Ok((JSON::Bool(*b), &tokens[1..])),
        Null => Ok((JSON::Null, &tokens[1..])),
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
            Ok((JSON::Object(obj), &slice[1..]))
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
            Ok((JSON::Array(arr), &slice[1..]))
        }
        _ => Err(JSONError::ParseError("Unexpected token in partial parse")),
    }
}

fn tokenize_input(s: &str) -> Result<Vec<&str>, JSONError> {
    let mut tokens: Vec<&str> = Vec::new();
    let control_chars = ['{', '}', '[', ']', ':', ','];

    let mut escaped = false;
    let mut in_string = false;
    let mut start_idx = 0;

    for (i, c) in s.chars().enumerate() {
        if !in_string && c.is_whitespace() {
            start_idx = i + 1;
            continue;
        }

        if c == '\\' {
            if !in_string {
                return Err(JSONError::ParseError("Unexpected escape character"));
            }
            if escaped {
                escaped = false;
            } else {
                escaped = true;
                continue;
            }
        }

        if c == '"' {
            if !in_string {
                in_string = true;
                start_idx = i;
            } else {
                in_string = false;
                tokens.push(&s[start_idx..i + 1]);
                start_idx = i + 1;
            }
            continue;
        }

        if in_string {
            continue;
        }

        if control_chars.contains(&c) {
            if start_idx < i {
                tokens.push(&s[start_idx..i]);
            }
            tokens.push(&s[i..i + 1]);
            start_idx = i + 1;
        }
    }

    Ok(tokens)
}

enum NodeMetadata<'a> {
    Key(&'a str),
    Object,
    Array,
    Literal,
    NeedKey,
}

struct Node<'a> {
    children: Vec<Rc<RefCell<Node<'a>>>>,
    metadata: NodeMetadata<'a>,
    value: Option<JSON>,
}

impl<'a> Node<'a> {
    fn get_children(&self) -> &Vec<Rc<RefCell<Node<'a>>>> {
        &self.children
    }

    fn add_child(&mut self, node: Node<'a>) -> Rc<RefCell<Node<'a>>> {
        let wrapped = Rc::new(RefCell::new(node));
        self.children.push(wrapped.clone());
        wrapped
    }

    fn new(metadata: NodeMetadata, value: Option<JSON>) -> Node {
        Node {
            children: Vec::new(),
            metadata,
            value,
        }
    }
}

struct Tree<'a> {
    head: Node<'a>,
}

impl<'a> Tree<'a> {
    fn get_head(&self) -> &Node<'a> {
        &self.head
    }
}

struct TreeIterator<'a> {
    stack: Vec<Rc<RefCell<Node<'a>>>>,
}

impl<'a> TreeIterator<'a> {
    fn new(tree: Tree<'a>) -> TreeIterator<'a> {
        let top = tree.head;
        let vec = vec![Rc::new(RefCell::new(top))];
        TreeIterator { stack: vec }
    }
}

impl<'a> Iterator for TreeIterator<'a> {
    type Item = Rc<RefCell<Node<'a>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let top = self.stack.pop();
        if top.is_none() {
            return top;
        }

        loop {
            let new_top = self.stack.last().unwrap().clone();
            let top_node = (*new_top).borrow();
            let children = top_node.get_children();
            if children.is_empty() {
                break;
            }
            self.stack.extend_from_slice(children);
        }

        top
    }
}

impl<'a> IntoIterator for Tree<'a> {
    type Item = Rc<RefCell<Node<'a>>>;
    type IntoIter = TreeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TreeIterator::new(self)
    }
}

impl FromStr for JSON {
    type Err = JSONError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //TODO: Finish collection at bottom, and reason how NeedKey is consumed
        let tokens = tokenize_input(s)?;

        let nodes = Vec::new();
        let current_scope = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            match *token {
                "{" => {
                    let obj_node = Node::new(NodeMetadata::Object, None);
                    let elem_node = Node::new(NodeMetadata::NeedKey, None);
                    let elem_node = obj_node.add_child(elem_node);
                    current_scope.push(Rc::new(RefCell::new(obj_node)));
                    current_scope.push(elem_node);
                }
                ":" => {
                    let scope = current_scope.last();
                    if scope.is_none() {
                        return Err(JSONError::ParseError("Unexpected colon"));
                    }

                    let scope = scope.unwrap();
                    let node = (*scope).borrow();
                    match node.metadata {
                        NodeMetadata::Key(_) => (),
                        _ => return Err(JSONError::ParseError("Unexpected colon")),
                    }
                }
                "}" => {
                    let scope = current_scope.pop();
                    if scope.is_none() {
                        return Err(JSONError::ParseError("Unexpected end curly brace"));
                    }

                    let scope = scope.unwrap();
                    let node = (*scope).borrow();

                    match node.metadata {
                        NodeMetadata::Object => (),
                        _ => return Err(JSONError::ParseError("Unexpected end curly brace")),
                    }

                    nodes.push(scope);
                }
                "[" => {
                    let arr_node = Node::new(NodeMetadata::Array, None);
                    current_scope.push(Rc::new(RefCell::new(arr_node)));
                }
                "," => {
                    let scope = current_scope.last();
                    match scope {
                        Some(node_wr) => match (*node_wr).borrow().metadata {
                            NodeMetadata::Array => (),
                            NodeMetadata::Object => {
                                let node = Node::new(NodeMetadata::NeedKey, None);
                                current_scope.push(Rc::new(RefCell::new(node)));
                            }
                            _ => return Err(JSONError::ParseError("Unexpected comma")),
                        },
                        _ => return Err(JSONError::ParseError("Unexpected comma")),
                    }
                }
                "]" => {
                    let scope = current_scope.pop();
                    match scope {
                        None => return Err(JSONError::ParseError("Unexpected end square brace")),
                        Some(rc) => match (*rc).borrow().metadata {
                            NodeMetadata::Array => {
                                nodes.push(scope.unwrap());
                            }
                            _ => return Err(JSONError::ParseError("Unexpected end square brace")),
                        },
                    }
                }
                st => {
                    let parent = current_scope.last();
                    let (json_val, error_str) = match st {
                        "true" => (JSON::Bool(true), "Unexpected boolean literal"),
                        "false" => (JSON::Bool(false), "Unexpected boolean literal"),
                        "null" => (JSON::Null, "Unexpected null value"),
                        _ => {
                            if let Ok(num) = st.parse::<f64>() {
                                (JSON::Number(num), "Unexpected number")
                            } else {
                                (JSON::String(st.to_string()), "Unexpected string")
                            }
                        }
                    };

                    match parent {
                        None => return Err(JSONError::ParseError(error_str)),
                        Some(rc) => {
                            let node = (*rc).borrow_mut();
                            match (json_val, node.metadata) {
                                (
                                    JSON::Bool(_) | JSON::Number(_) | JSON::Null,
                                    NodeMetadata::Key(_) | NodeMetadata::Array,
                                )
                                | (
                                    JSON::String(_),
                                    NodeMetadata::NeedKey
                                    | NodeMetadata::Key(_)
                                    | NodeMetadata::Array,
                                ) => {
                                    let new_node = Node::new(NodeMetadata::Literal, Some(json_val));
                                    let new_node = node.add_child(new_node);
                                    nodes.push(new_node);
                                }
                                _ => return Err(JSONError::ParseError(error_str)),
                            }
                        }
                    }
                }
            }
        }

        let tree_head = current_scope.pop();
        if tree_head.is_none() {
            return Err(JSONError::ParseError(
                "Extremely unexpected parsing error, everything consumed?",
            ));
        }
        let tree_head = tree_head.unwrap();

        let parent_node: Node = (*tree_head).into_inner();
        let tree = Tree { head: parent_node };

        for node in tree {
            let n = (*node).borrow_mut();
            if n.value.is_some() {
                continue;
            }

            match n.metadata {
                Key(_) => {}
            }
        }
    }
}
