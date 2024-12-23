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

fn tokenize_input(s: &str) -> Result<(Vec<&str>, usize), JSONError> {
    let mut commas = 0;
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
            if c == ',' {
                commas += 1;
            }
            if start_idx < i {
                tokens.push(&s[start_idx..i]);
            }
            tokens.push(&s[i..i + 1]);
            start_idx = i + 1;
        }
    }

    Ok((tokens, commas))
}

#[derive(Debug)]
enum NodeMetadata<'a> {
    Object(Vec<&'a str>),
    Array,
    Literal,
    Default,
}
#[derive(Debug)]
struct Node<'a> {
    children: Vec<Rc<RefCell<Node<'a>>>>,
    metadata: NodeMetadata<'a>,
    value: Option<JSON>,
}

impl<'a> Node<'a> {
    fn get_children(&self) -> &Vec<Rc<RefCell<Node<'a>>>> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<Rc<RefCell<Node<'a>>>> {
        &mut self.children
    }

    fn add_child_wrapped(&mut self, node: Rc<RefCell<Node<'a>>>) {
        self.children.push(node)
    }

    fn new(metadata: NodeMetadata, value: Option<JSON>) -> Node {
        Node {
            children: Vec::new(),
            metadata,
            value,
        }
    }
}

impl<'a> Default for Node<'a> {
    fn default() -> Node<'a> {
        Node {
            children: Vec::new(),
            metadata: NodeMetadata::Default,
            value: None,
        }
    }
}

fn add_to_top<'a>(
    vect: &mut Vec<Rc<RefCell<Node<'a>>>>,
    child_node: Rc<RefCell<Node<'a>>>,
    err_str: &'static str,
) -> Result<(), JSONError> {
    let top_node = vect.last();
    let cur_node = (*child_node).borrow();
    match top_node {
        Some(rc) => {
            let mut top_node = (*rc).borrow_mut();
            match (&cur_node.metadata, &top_node.metadata) {
                (
                    NodeMetadata::Object(_) | NodeMetadata::Array | NodeMetadata::Literal,
                    NodeMetadata::Object(_) | NodeMetadata::Array | NodeMetadata::Default,
                ) => {
                    top_node.add_child_wrapped(child_node.clone());
                    Ok(())
                }
                (_, _) => Err(JSONError::ParseError(err_str)),
            }
        }
        None => Ok(()),
    }
}

fn tree_from_tokens(
    tokens: Vec<&str>,
    node_size_hint: Option<usize>,
) -> Result<Vec<Rc<RefCell<Node>>>, JSONError> {
    let mut nodes = Vec::with_capacity(node_size_hint.unwrap_or(0));
    let top_node = Node::new(NodeMetadata::Default, None);
    let top_node_ref = Rc::new(RefCell::new(top_node));
    let mut current_scope: Vec<Rc<RefCell<Node>>> = vec![top_node_ref.clone()];
    let mut next_is_key = false;
    drop(top_node_ref);
    for token in tokens.iter() {
        match *token {
            "{" => {
                let obj_node = Node::new(NodeMetadata::Object(Vec::new()), None);
                let wrapped_obj_node = Rc::new(RefCell::new(obj_node));
                add_to_top(
                    &mut current_scope,
                    wrapped_obj_node.clone(),
                    "Unexpected start of object",
                )?;
                nodes.push(wrapped_obj_node.clone());
                current_scope.push(wrapped_obj_node);
                next_is_key = true;
            }
            ":" => {
                let scope = current_scope.last();
                if scope.is_none() {
                    return Err(JSONError::ParseError("Unexpected colon"));
                }

                let scope = scope.unwrap();
                let node = (*scope).borrow();
                match node.metadata {
                    NodeMetadata::Object(_) => (),
                    _ => return Err(JSONError::ParseError("Unexpected colon")),
                }
                next_is_key = false;
            }
            "}" => {
                let scope = current_scope.pop();
                if scope.is_none() {
                    return Err(JSONError::ParseError("Unexpected end curly brace"));
                }

                let scope = scope.unwrap();
                let node = (*scope).borrow();

                match node.metadata {
                    NodeMetadata::Object(_) => (),
                    _ => return Err(JSONError::ParseError("Unexpected end curly brace")),
                }
            }
            "[" => {
                let arr_node = Node::new(NodeMetadata::Array, None);
                let wrapped_arr_node = Rc::new(RefCell::new(arr_node));
                add_to_top(
                    &mut current_scope,
                    wrapped_arr_node.clone(),
                    "Unexpected start of array",
                )?;
                nodes.push(wrapped_arr_node.clone());
                current_scope.push(wrapped_arr_node);
            }
            "," => {
                let scope = current_scope.last();
                match scope {
                    Some(node_wr) => match (*node_wr).borrow().metadata {
                        NodeMetadata::Array => (),
                        NodeMetadata::Object(_) => {
                            next_is_key = true;
                        }
                        _ => return Err(JSONError::ParseError("Unexpected comma")),
                    },
                    _ => return Err(JSONError::ParseError("Unexpected comma")),
                };
            }
            "]" => {
                let scope = current_scope.pop();
                match scope {
                    None => return Err(JSONError::ParseError("Unexpected end square brace")),
                    Some(rc) => match (*rc).borrow().metadata {
                        NodeMetadata::Array => (),
                        _ => return Err(JSONError::ParseError("Unexpected end square brace")),
                    },
                }
            }
            st => {
                let (json_val, error_str) = match st {
                    "true" => (JSON::Bool(true), "Unexpected boolean literal"),
                    "false" => (JSON::Bool(false), "Unexpected boolean literal"),
                    "null" => (JSON::Null, "Unexpected null value"),
                    _ => {
                        if let Ok(num) = st.parse::<f64>() {
                            (JSON::Number(num), "Unexpected number")
                        } else {
                            (
                                JSON::String(st[1..st.len() - 1].to_string()),
                                "Unexpected string",
                            )
                        }
                    }
                };

                if next_is_key {
                    let parent = current_scope.last();
                    match parent {
                        None => (),
                        Some(rc) => {
                            let mut node = (*rc).borrow_mut();
                            match (&node.metadata, &json_val) {
                                (NodeMetadata::Object(_), JSON::String(_)) => {
                                    let md = &mut node.metadata;
                                    if let NodeMetadata::Object(keys) = md {
                                        keys.push(&st[1..st.len() - 1]);
                                    }
                                    continue;
                                }
                                (NodeMetadata::Object(_), _) => {
                                    return Err(JSONError::ParseError(
                                        "Non string used as object key",
                                    ))
                                }
                                _ => {
                                    return Err(JSONError::ParseError(
                                        "Tried to add key to non-object",
                                    ))
                                }
                            }
                        }
                    }
                }
                let node = Node::new(NodeMetadata::Literal, Some(json_val));
                let wrapped_node = Rc::new(RefCell::new(node));
                nodes.push(wrapped_node.clone());
                add_to_top(&mut current_scope, wrapped_node, error_str)?;
            }
        }
    }

    if current_scope.len() > 1 {
        return Err(JSONError::ParseError(
            "More than one independent JSON object detected",
        ));
    }
    let tree_head = current_scope.pop();
    if tree_head.is_none() {
        return Err(JSONError::ParseError(
            "Extremely unexpected parsing error, everything consumed?",
        ));
    }
    Ok(nodes)
}

fn consume_tree(mut node_order: Vec<Rc<RefCell<Node>>>) -> Result<JSON, JSONError> {
    node_order.reverse();
    let mut iter = node_order.drain(..);
    let parsed_json = loop {
        let node = iter
            .next()
            .expect("Should break at bottom, non child node is root");
        let mut n = (*node).borrow_mut();

        if n.value.is_some() {
            continue;
        }

        match &n.metadata {
            NodeMetadata::Default => {
                let children: &mut Vec<Rc<RefCell<Node<'_>>>> = n.get_children_mut();
                if children.len() != 1 {
                    return Err(JSONError::ParseError(
                        "Keyed object has more than one child",
                    ));
                }

                let val_node_rc = children.pop().expect("Has 1 child");
                let val_node = Rc::into_inner(val_node_rc)
                    .expect("Should be only child")
                    .into_inner();
                n.value = val_node.value;
            }
            NodeMetadata::Array => {
                let children = n.get_children_mut();
                let mut json_vs = Vec::with_capacity(children.len());
                let mut err = false;

                children.drain(..).for_each(|child| {
                    let child_node = Rc::into_inner(child)
                        .expect("Should be only child now")
                        .into_inner();
                    if let Some(js) = child_node.value {
                        json_vs.push(js);
                    } else {
                        err = true;
                    }
                });
                if err {
                    return Err(JSONError::ParseError("Unparsed child of array object"));
                }
                n.value = Some(JSON::Array(json_vs))
            }
            NodeMetadata::Object(keys) => {
                let immut_children = n.get_children();
                if immut_children.len() != keys.len() {
                    return Err(JSONError::ParseError("Unkeyed child of object"));
                }
                let mut json_ob = HashMap::with_capacity(immut_children.len());

                let mut key_strs: Vec<String> = keys.iter().map(|s| String::from(*s)).collect();
                let children = n.get_children_mut();
                let mut err = false;
                let mut err_str = "";
                let drain_iter = children.drain(..);
                let zipped_iter = drain_iter.zip(key_strs.drain(..));
                zipped_iter.for_each(|(child, key)| {
                    let child_node = Rc::into_inner(child)
                        .expect("Should be only child")
                        .into_inner();
                    if child_node.value.is_none() {
                        err = true;
                        err_str = "Unparsed child of object";
                    }
                    let child_val = child_node.value.unwrap();
                    json_ob.insert(key, child_val);
                });

                if err {
                    return Err(JSONError::ParseError(err_str));
                }

                n.value = Some(JSON::Object(json_ob))
            }
            NodeMetadata::Literal => continue,
        }

        if Rc::strong_count(&node) == 1 {
            drop(n);
            let inner_node = Rc::into_inner(node)
                .expect("Guaranteed to be zero")
                .into_inner();
            break inner_node.value;
        }
    };
    match iter.next() {
        None => (),
        Some(_) => {
            return Err(JSONError::ParseError(
                "Multiple independent JSON objects present",
            ))
        }
    }
    if let Some(js) = parsed_json {
        Ok(js)
    } else {
        Err(JSONError::ParseError("Parsing failed??"))
    }
}

impl FromStr for JSON {
    type Err = JSONError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tokens, commas) = tokenize_input(s)?;

        let nodes = tree_from_tokens(tokens, Some(commas))?;
        consume_tree(nodes)
    }
}
