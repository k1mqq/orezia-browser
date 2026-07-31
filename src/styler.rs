use std::collections::HashMap;

use crate::{
    css_parser::{self, Selector, SimpleSelector, StyleSheet, Unit, Value},
    html_parser::{Dom, NodeId, NodeType},
};

pub struct StyledTree {
    pub nodes: Vec<StyledNode>,
}

pub struct StyledNode {
    pub dom_node_type: NodeType,
    pub styles: HashMap<String, Value>,
    pub children: Vec<StyledNodeId>,
}

// pub enum StyledNodeType {
//     Element {
//         tag: String,
//         attributes: Vec<(String, String)>,
//     },
//     Text(String),
// }

// #[derive(Clone, Debug)]
// pub enum StyleValue {
//     Keyword(String),
//     Length(f32, Unit),
//     ColorValue(Color),
// }

// #[derive(Clone, Debug)]
// pub enum Unit {
//     Px,
//     // Percent,
// }

// #[derive(Clone, Debug)]
// pub struct Color {
//     r: u8,
//     g: u8,
//     b: u8,
//     a: u8,
// }

type StyledNodeId = usize;

impl StyledNode {
    pub fn get_text(&self) -> Option<&String> {
        if let NodeType::Text(t) = &self.dom_node_type {
            return Some(t);
        } else {
            return None;
        }
    }
}

impl StyledTree {
    pub fn build(dom: &Dom) -> StyledTree {
        let mut nodes = Vec::new();

        let style_sheets = extract_stylesheets(dom);

        let body_id = dom
            .get_element_by_tag_name("body")
            .first()
            // is this clone ok?
            .cloned()
            .expect("no body element?");

        next_node(&mut nodes, dom, body_id, None, &style_sheets);

        Self { nodes }
    }
}

fn extract_stylesheets(dom: &Dom) -> Vec<StyleSheet> {
    let style_ids = dom.get_element_by_tag_name("style");

    style_ids
        .iter()
        .filter_map(|&id| {
            let children = &dom.nodes[id].children;
            if children.len() == 1 {
                if let NodeType::Text(t) = &dom.nodes[children[0]].node_type {
                    return Some(css_parser::parse(t));
                }
            }
            return None;
        })
        .collect()
}

fn next_node(
    nodes: &mut Vec<StyledNode>,
    dom: &Dom,
    node_id: NodeId,
    parent_id: Option<StyledNodeId>,
    style_sheets: &Vec<StyleSheet>,
) -> StyledNodeId {
    let node = &dom.nodes[node_id];
    let id = nodes.len();
    let mut styles = match parent_id {
        Some(parent) => {
            nodes[parent].children.push(id);

            nodes[parent]
                .styles
                .iter()
                .filter(|(k, _)| is_inheritable(k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }
        None => HashMap::new(),
    };

    match &node.node_type {
        NodeType::Element { tag, attributes } => {
            style_by_tag_name(&mut styles, tag, style_sheets);
            style_by_attribute(&mut styles, attributes, style_sheets);
        }
        NodeType::Text(_) => {
            styles.insert("display".to_string(), Value::Keyword("inline".to_string()));
        }
        _ => {}
    }

    nodes.push(StyledNode {
        dom_node_type: node.node_type.clone(),
        styles: styles,
        children: Vec::new(),
    });

    for child in &node.children {
        next_node(nodes, dom, *child, Some(id), style_sheets);
    }

    id
}

fn style_by_tag_name(
    styles: &mut HashMap<String, Value>,
    tag_name: &str,
    style_sheets: &Vec<StyleSheet>,
) {
    // elements' default
    match tag_name {
        "script" | "style" => {
            styles.insert("display".to_string(), Value::Keyword("none".to_string()));
        }
        "body" => {
            styles.insert("margin".to_string(), Value::Length(8.0, Unit::Px));
        }
        "a" | "span" => {
            styles.insert("display".to_string(), Value::Keyword("inline".to_string()));
        }
        _ => {}
    }

    // apply stylesheets

    // it took 3h for this code
    // let declarations: Vec<&css_parser::Declaration> =
    style_sheets
        .iter()
        .flat_map(|ss| &ss.rules)
        .filter(|r| {
            r.selectors.iter().any(|selector| {
                if let Selector::Simple(s) = selector {
                    if let Some(tag) = &s.tag_name {
                        return tag == tag_name;
                    }
                }
                false
            })
        })
        .flat_map(|r| r.declarations.iter())
        .for_each(|d| {
            styles.insert(d.name.clone(), d.value.clone());
        });
}

fn style_by_attribute(
    styles: &mut HashMap<String, Value>,
    attributes: &Vec<(String, String)>,
    style_sheets: &Vec<StyleSheet>,
) {
    for (attr_key, attr_value) in attributes {
        match attr_key.as_str() {
            "id" => {
                style_sheets
                    .iter()
                    .flat_map(|ss| &ss.rules)
                    .filter(|r| {
                        r.selectors.iter().any(|selector| {
                            if let Selector::Simple(s) = selector {
                                if let Some(id) = &s.id {
                                    return id == attr_value;
                                }
                            }
                            false
                        })
                    })
                    .flat_map(|r| r.declarations.iter())
                    .for_each(|d| {
                        styles.insert(d.name.clone(), d.value.clone());
                    });
            }

            "class" => {
                style_sheets
                    .iter()
                    .flat_map(|ss| &ss.rules)
                    .filter(|r| {
                        r.selectors.iter().any(|selector| {
                            if let Selector::Simple(s) = selector {
                                return s.class.iter().any(|c| c == attr_value);
                            }
                            false
                        })
                    })
                    .flat_map(|r| r.declarations.iter())
                    .for_each(|d| {
                        styles.insert(d.name.clone(), d.value.clone());
                    });
            }
            _ => {}
        }
    }
}

fn is_inheritable(key: &str) -> bool {
    !matches!(key, "margin" | "padding" | "width" | "height" | "display")
}
