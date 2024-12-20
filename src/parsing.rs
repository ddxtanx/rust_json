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

#[derive(Debug)]
enum NodeMetadata<'a> {
    Key(&'a str),
    Object,
    Array,
    Literal,
    NeedKey,
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

    fn add_child(&mut self, node: Node<'a>) -> Rc<RefCell<Node<'a>>> {
        let wrapped = Rc::new(RefCell::new(node));
        self.children.push(wrapped.clone());
        wrapped
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

struct TreeIterator<'a> {
    stack: Vec<(Rc<RefCell<Node<'a>>>, bool)>,
}

impl<'a> TreeIterator<'a> {
    fn new(tree: Rc<RefCell<Node<'a>>>) -> TreeIterator<'a> {
        let vec = vec![(tree, false)];
        TreeIterator { stack: vec }
    }
}

impl<'a> Iterator for TreeIterator<'a> {
    type Item = Rc<RefCell<Node<'a>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let top = self.stack.last_mut();
        if top.is_none() {
            return None;
        }

        loop {
            let (top_node_rc, childed) = self.stack.last_mut().expect("Should never be None");
            let mut top_node = (*top_node_rc).borrow_mut();
            let children = top_node.get_children_mut();
            if *childed || children.is_empty() {
                break;
            }

            *childed = true;
            let falsed_children: Vec<(Rc<RefCell<Node>>, bool)> =
                children.iter().map(|n| (n.clone(), false)).collect();
            drop(top_node);
            self.stack.extend(falsed_children);
        }

        self.stack.pop().map(|(n, _)| n)
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
                    NodeMetadata::Object | NodeMetadata::Array | NodeMetadata::Literal,
                    NodeMetadata::Key(_),
                ) => {
                    top_node.add_child_wrapped(child_node.clone());
                    drop(top_node);
                    vect.pop();
                    Ok(())
                }
                (
                    NodeMetadata::Object | NodeMetadata::Array | NodeMetadata::Literal,
                    NodeMetadata::Object | NodeMetadata::Array | NodeMetadata::Default,
                ) => {
                    top_node.add_child_wrapped(child_node.clone());
                    Ok(())
                }
                (NodeMetadata::Key(_) | NodeMetadata::NeedKey, NodeMetadata::Object) => {
                    top_node.add_child_wrapped(child_node.clone());
                    Ok(())
                }
                (_, _) => Err(JSONError::ParseError(err_str)),
            }
        }
        None => Ok(()),
    }
}

impl FromStr for JSON {
    type Err = JSONError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //TODO: Finish collection at bottom, and reason how NeedKey is consumed
        let tokens = tokenize_input(s)?;

        let top_node = Node::new(NodeMetadata::Default, None);
        let top_node_ref = Rc::new(RefCell::new(top_node));
        let mut current_scope: Vec<Rc<RefCell<Node>>> = vec![top_node_ref.clone()];
        drop(top_node_ref);
        for token in tokens.iter() {
            match *token {
                "{" => {
                    let mut obj_node = Node::new(NodeMetadata::Object, None);
                    let elem_node = Node::new(NodeMetadata::NeedKey, None);
                    let elem_node = obj_node.add_child(elem_node);
                    let wrapped_obj_node = Rc::new(RefCell::new(obj_node));
                    add_to_top(
                        &mut current_scope,
                        wrapped_obj_node.clone(),
                        "Unexpected start of object",
                    )?;
                    current_scope.push(wrapped_obj_node);
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
                }
                "[" => {
                    let arr_node = Node::new(NodeMetadata::Array, None);
                    let wrapped_arr_node = Rc::new(RefCell::new(arr_node));
                    add_to_top(
                        &mut current_scope,
                        wrapped_arr_node.clone(),
                        "Unexpected start of array",
                    )?;
                    current_scope.push(wrapped_arr_node);
                }
                "," => {
                    let scope = current_scope.last();
                    let new_node = match scope {
                        Some(node_wr) => match (*node_wr).borrow().metadata {
                            NodeMetadata::Array => None,
                            NodeMetadata::Object => {
                                let node = Node::new(NodeMetadata::NeedKey, None);
                                Some(Rc::new(RefCell::new(node)))
                            }
                            _ => return Err(JSONError::ParseError("Unexpected comma")),
                        },
                        _ => return Err(JSONError::ParseError("Unexpected comma")),
                    };
                    if let Some(new_node) = new_node {
                        add_to_top(&mut current_scope, new_node.clone(), "Unexpected comma")?;
                        current_scope.push(new_node);
                    }
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

                    let parent = current_scope.last();
                    match parent {
                        None => (),
                        Some(rc) => {
                            let mut node = (*rc).borrow_mut();
                            match (&node.metadata, &json_val) {
                                (NodeMetadata::NeedKey, JSON::String(_)) => {
                                    node.metadata = NodeMetadata::Key(&st[1..st.len() - 1]);
                                    continue;
                                }
                                (NodeMetadata::NeedKey, _) => {
                                    return Err(JSONError::ParseError(
                                        "Non string used as object key",
                                    ))
                                }
                                _ => {}
                            }
                        }
                    }
                    let node = Node::new(NodeMetadata::Literal, Some(json_val));
                    let wrapped_node = Rc::new(RefCell::new(node));
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
        let tree_head = tree_head.unwrap();

        let mut iter = TreeIterator::new(tree_head);
        let parsed_json = loop {
            let node = iter
                .next()
                .expect("Should break at bottom, non child node is root");
            let mut n = (*node).borrow_mut();

            if n.value.is_some() {
                continue;
            }

            match n.metadata {
                NodeMetadata::Key(_) | NodeMetadata::Default => {
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
                    let mut json_vs = Vec::new();
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
                NodeMetadata::Object => {
                    let children = n.get_children_mut();
                    let mut json_ob = HashMap::new();
                    let mut err = false;
                    let mut err_str = "";
                    children.drain(..).for_each(|child| {
                        let child_node = Rc::into_inner(child)
                            .expect("Should be only child")
                            .into_inner();
                        match (child_node.metadata, child_node.value) {
                            (NodeMetadata::Key(s), Some(js)) => {
                                json_ob.insert(String::from(s), js);
                            }
                            (_, None) => {
                                err = true;
                                err_str = "Unparsed child of object";
                            }
                            (_, _) => {
                                err = true;
                                err_str = "Non Keyed child of object";
                            }
                        }
                    });

                    if err {
                        return Err(JSONError::ParseError(err_str));
                    }

                    n.value = Some(JSON::Object(json_ob))
                }
                NodeMetadata::Literal => continue,
                _ => return Err(JSONError::ParseError("Unexpected node in parse tree")),
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
}
