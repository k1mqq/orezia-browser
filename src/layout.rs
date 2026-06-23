use crate::html_parser::{Dom, NodeId, NodeType};

pub struct Layout {
    pub components: Vec<Component>,
}

pub struct Component {
    pub dimentions: Dimentions,
    pub text: Option<String>,

}

#[derive(Default)]
pub struct Dimentions {
    pub content: Rect,
    pub padding: EdgeSize,
    pub border: EdgeSize,
    pub margin: EdgeSize,
}

type ComponentId = usize;

// pub enum Content {
//     Text(String),
// }

#[derive(Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Default)]
pub struct EdgeSize {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
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

fn next_component(components: &mut Vec<Component>, dom: &Dom, node_id: NodeId, parent: Option<ComponentId>, brother: Option<ComponentId>) -> Option<ComponentId> {
    // println!("{}", node_id);
    let node = &dom.nodes[node_id];
    let id = components.len();


    let mut y = match parent {
        Some(parent_id) => {
            components[parent_id].dimentions.content.y + components[parent_id].dimentions.padding.top
        }
        None => 0.0
    };

    match brother {
        Some(brother_id) => {
            y += components[brother_id].dimentions.content.y + components[brother_id].dimentions.content.height + components[brother_id].dimentions.border.bottom + components[brother_id].dimentions.margin.bottom
        }
        None => {}
    }

    match &node.node_type {
        NodeType::Element { tag, attributes } => {
            if matches!(tag.as_str(), "script" | "style"){
                return None;
            }
            components.push(Component {
                dimentions: Dimentions {
                    content: Rect { x: 0.0, y: y, width: 100.0, height: 20.0 },
                    padding: EdgeSize::default(),
                    border: EdgeSize::default(),
                    margin: EdgeSize::default(),
                },
                text: None,
            });
        }
        NodeType::Text(text) => {
            components.push(Component {
                dimentions: Dimentions {
                    content: Rect { x: 0.0, y: y, width: 100.0, height: 20.0 },
                    padding: EdgeSize::default(),
                    border: EdgeSize::default(),
                    margin: EdgeSize::default(),
                },
                text: Some(text.to_string()),
            });
        }
        _ => {
            // ;)
        }
    }

    let mut last_child = None;
    let mut height = 40.0;
    for child_node in &node.children {
        let Some(child) = next_component(components, dom, *child_node, Some(id), last_child) else {
            continue;
        };
        height += components.last().unwrap().dimentions.margin.bottom + components.last().unwrap().dimentions.content.height;
        last_child = Some(child);
    }

    components[id].dimentions.content.height = height;

    return Some(id);
}