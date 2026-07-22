use std::collections::HashMap;

use crate::{
    html_parser::{Dom, NodeId, NodeType},
    styler::StyleValue::{Keyword, Length},
};

pub struct StyledTree {
    pub nodes: Vec<StyledNode>,
}

pub struct StyledNode {
    pub dom_node_type: NodeType,
    pub styles: HashMap<String, StyleValue>,
    pub children: Vec<StyledNodeId>,
}

// pub enum StyledNodeType {
//     Element {
//         tag: String,
//         attributes: Vec<(String, String)>,
//     },
//     Text(String),
// }

#[derive(Clone, Debug)]
pub enum StyleValue {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

#[derive(Clone, Debug)]
pub enum Unit {
    Px,
    // Percent,
}

#[derive(Clone, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

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

        let body_id = dom
            .nodes
            .iter()
            .position(|node| {
                if let NodeType::Element { tag, attributes: _ } = &node.node_type {
                    tag == "body"
                } else {
                    false
                }
            })
            .expect("no body element?");

        next_node(&mut nodes, dom, body_id, None);

        Self { nodes }
    }
}

fn next_node(
    nodes: &mut Vec<StyledNode>,
    dom: &Dom,
    node_id: NodeId,
    parent_id: Option<StyledNodeId>,
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
            match tag.as_str() {
                "script" | "style" => {
                    styles.insert(
                        "display".to_string(),
                        StyleValue::Keyword("none".to_string()),
                    );
                }
                "body" => {
                    styles.insert("margin".to_string(), StyleValue::Length(8.0, Unit::Px));
                }
                "a" | "span" => {
                    styles.insert(
                        "display".to_string(),
                        StyleValue::Keyword("inline".to_string()),
                    );
                }
                _ => {}
            }
            if let Some((_, style_text)) =
                attributes.iter().find(|(attr_key, _)| attr_key == "style")
            {
                // nice iter:)
                let raw_styles: Vec<&str> = style_text.split(';').collect();
                for raw_style in raw_styles {
                    let Some((key, raw_value)) = raw_style.split_once(':') else {
                        continue;
                    };

                    if raw_value.ends_with("px") {
                        let Ok(value) = raw_value.replace("px", "").parse::<f32>() else {
                            continue;
                        };
                        styles.insert(key.to_string(), Length(value, Unit::Px));
                    // } else if raw_value.ends_with("%") {
                    //     let Ok(value) = raw_value.replace("%", "").parse::<f32>() else {
                    //         continue;
                    //     };
                    //     styles.insert(key.to_string(), Length(value, Unit::Percent));
                    } else {
                        styles.insert(key.to_string(), Keyword(raw_value.to_string()));
                    }
                }
            }
        }
        NodeType::Text(_) => {
            styles.insert(
                "display".to_string(),
                StyleValue::Keyword("inline".to_string()),
            );
        }
        _ => {}
    }

    nodes.push(StyledNode {
        dom_node_type: node.node_type.clone(),
        styles: styles,
        children: Vec::new(),
    });

    for child in &node.children {
        next_node(nodes, dom, *child, Some(id));
    }

    id
}

fn is_inheritable(key: &str) -> bool {
    !matches!(key, "margin" | "padding" | "width" | "height" | "display")
}
