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

        next_component(&mut components, dom, 0, None, None);

        Self {
            components: components,
        }
    }
}

fn next_component(components: &mut Vec<Component>, dom: &Dom, node_id: NodeId, parent: Option<ComponentId>, brother: Option<ComponentId>) -> ComponentId {
    // println!("{}", node_id);
    let node = &dom.nodes[node_id];
    let id = components.len();


    let mut y = match parent {
        Some(parent_id) => {
            components[parent_id].rect.y
        }
        None => 0.0
    };

    match brother {
        Some(brother_id) => {
            y += components[brother_id].rect.y + components[brother_id].rect.height;
        }
        None => {}
    }

    match &node.node_type {
        NodeType::Element { tag, attributes } => {
            components.push(Component {
                rect: Rect{ x: 0.0, y: y , width: 100.0, height: 40.0 },
                content: None,
            });
        }
        NodeType::Text(text) => {
             components.push(Component {
                rect: Rect{ x: 0.0, y: y, width: 100.0, height: 40.0 },
                content: Some(Content::Text(text.to_string())),

            });           
        }
        _ => {
            // ;)
        }
    }

    let mut last_child = None;
    let mut height = 40.0;
    for child_node in &node.children {
        let child = next_component(components, dom, *child_node, Some(id), last_child);
        height += components.last().unwrap().rect.height;
        last_child = Some(child);
    }

    components[id].rect.height = height;

    return id;
}