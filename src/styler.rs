use std::collections::HashMap;

use crate::{
    css_parser::{self, Color, Selector, StyleSheet, Unit, Value},
    html_parser::{Dom, Node, NodeId, NodeType},
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
    pub fn build(dom: &Dom, css: &Vec<StyleSheet>) -> StyledTree {
        let mut nodes = Vec::new();

        let mut style_sheets = extract_stylesheets(dom);
        style_sheets.extend(css.iter().cloned());

        let body_id = dom
            .get_element_by_tag_name("body")
            .first()
            // is this clone ok?
            .cloned()
            .expect("no body element?");

        next_node(&mut nodes, dom, body_id, None, &style_sheets);

        Self { nodes }
    }
    pub fn print(&self, node: StyledNodeId, depth: usize) {
        let indent = "  ".repeat(depth);
        match &self.nodes[node].dom_node_type {
            NodeType::Document => {}
            NodeType::Element { tag, attributes } => {
                println!("{}| <{}>", indent, tag);
                for (name, value) in attributes {
                    println!("{}|    {}=\"{}\"", indent, name, value);
                }
                for (style_key, style_value) in &self.nodes[node].styles {
                    println!("{}|    {}={:?}", indent, style_key, style_value);
                }
                for &child in &self.nodes[node].children {
                    self.print(child, depth + 1);
                }
            }
            NodeType::Text(t) => {
                println!("{}| \"{}\"", indent, t);
            }
        }
    }
}

fn extract_stylesheets(dom: &Dom) -> Vec<StyleSheet> {
    let mut ss = Vec::new();
    let style_ids = dom.get_element_by_tag_name("style");

    style_ids.iter().for_each(|&id| {
        let children = &dom.nodes[id].children;
        if children.len() == 1 {
            if let NodeType::Text(t) = &dom.nodes[children[0]].node_type {
                ss.push(css_parser::parse(t));
            }
        }
    });

    ss
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

    apply_style(&mut styles, &node, style_sheets);

    match &node.node_type {
        NodeType::Element { tag, attributes } => {
            // style_by_tag_name(&mut styles, tag, style_sheets);
            // style_by_attribute(&mut styles, attributes, style_sheets);
        }
        NodeType::Text(text) => {
            if text.trim().is_empty() {
                styles.insert("display".to_string(), Value::Keyword("none".to_string()));
            } else {
                styles.insert("display".to_string(), Value::Keyword("inline".to_string()));
            }
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

fn apply_style(styles: &mut HashMap<String, Value>, node: &Node, style_sheets: &Vec<StyleSheet>) {
    if let NodeType::Element { tag, attributes } = &node.node_type {
        match tag.as_str() {
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

        for (k, v) in attributes {
            if k == "bgcolor" {
                let hex = v.trim_start_matches('#');

                let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap();

                styles.insert(
                    "background-color".to_string(),
                    Value::ColorValue(Color { r, g, b, a: 255 }),
                );
            }
        }
    }
    style_sheets
        .iter()
        .flat_map(|ss| &ss.rules)
        .filter(|r| r.selectors.iter().any(|selector| selector.matches(node)))
        .flat_map(|r| r.declarations.iter())
        .for_each(|d| {
            styles.insert(d.name.clone(), d.value.clone());
        });
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
