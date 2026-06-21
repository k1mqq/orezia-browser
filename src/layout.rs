use crate::html_parser::{Dom, NodeId, NodeType};

pub struct Layout {
    pub components: Vec<Component>,
}

pub struct Component {
    pub rect: Rect,
    pub content: Option<Content>,

}

type ComponentId = usize;

pub enum Content {
    Text(String),
}

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Layout {
    pub fn build(dom: &Dom) -> Self {
        let mut components = Vec::new();

        next_component(&mut components, dom, 0, None);

        Self {
            components: components,
        }
    }
}

fn next_component(components: &mut Vec<Component>, dom: &Dom, node_id: NodeId, parent: Option<ComponentId>) -> ComponentId {
    // println!("{}", node_id);
    let node = &dom.nodes[node_id];
    match &node.node_type {
        // this is temporary!!
        NodeType::Document => {
            // no Document node exists.
            return 0;
        },
        NodeType::Element{ tag, attributes} => {
            let id = components.len();
            let mut children = Vec::new();


            let y = match parent {
                Some(parent_id) => {
                    components[parent_id].rect.y + 40.0
                }
                None => 0.0
            };
            components.push(Component {
                rect: Rect{ x: 0.0, y: y , width: 100.0, height: 20.0 },
                content: None,

            });

            for child in &node.children {
                children.push(next_component(components, dom, *child, Some(id)));
            }

            return id;
        }
        NodeType::Text(text) => {
            let id = components.len();
            let mut children = Vec::new();
            let y = match parent {
                Some(parent_id) => {
                    components[parent_id].rect.y + 40.0
                }
                None => 0.0
            };
            components.push(Component {
                rect: Rect{ x: 0.0, y: y, width: 100.0, height: 20.0 },
                content: Some(Content::Text(text.to_string())),

            });

            for child in &node.children {
                children.push(next_component(components, dom, *child, Some(id)));
            }

            return id;
        }
    }
}