use std::collections::HashMap;

use crate::{
    html_parser::{self, Dom, Node, NodeId, NodeType},
    styler::StyleValue::{Keyword, Length},
};

pub struct StyledTree {
    pub nodes: Vec<StyledNode>,
}

pub struct StyledNode {
    pub node: NodeType,
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

#[derive(Clone)]
pub enum StyleValue {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

#[derive(Clone)]
pub enum Unit {
    Px,
    // Percent,
}

#[derive(Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

type StyledNodeId = usize;

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
    parent: Option<StyledNodeId>,
) -> StyledNodeId {
    let node = &dom.nodes[node_id];
    let id = nodes.len();
    let mut styles = match parent {
        Some(parent) => nodes[parent].styles.clone(),
        None => HashMap::new(),
    };
    let mut children_id = Vec::new();

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
        NodeType::Text(text) => {}
    }

    for child in &node.children {
        children_id.push(next_node(nodes, dom, *child, Some(id)));
    }

    nodes.push(StyledNode {
        node: node.node_type.clone(),
        styles: styles,
        children: children_id,
    });

    id
}
