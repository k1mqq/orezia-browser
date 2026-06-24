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

        let body_id = dom.nodes.iter().position(|node|
            if let NodeType::Element { tag, attributes: _ } = &node.node_type {
                tag == "body"
            } else {
                false
            }
        ).expect("no body element?");

        next_component(&mut components, dom, body_id, None, None);

        Self {
            components: components,
        }
    }
}

impl Dimentions {
    fn inner_width(&self) -> f32 {
        self.content.width + self.padding.left + self.padding.right
    }

    fn inner_height(&self) -> f32 {
        self.content.height + self.padding.top + self.padding.bottom
    }

    fn outer_width(&self) -> f32 {
        self.inner_width() + self.border.left + self.border.right + self.margin.left + self.margin.right
    }

    fn outer_height(&self) -> f32 {
        self.inner_height() + self.border.top + self.border.bottom + self.margin.top + self.margin.bottom
    }
}

fn next_component(components: &mut Vec<Component>, dom: &Dom, node_id: NodeId, parent: Option<ComponentId>, brother: Option<ComponentId>) -> Option<ComponentId> {
    // println!("{}", node_id);
    let node = &dom.nodes[node_id];
    let id = components.len();

    let (x, y) = match brother {
        Some(brother_id) => {(
            components[brother_id].dimentions.content.x,
            components[brother_id].dimentions.content.y + components[brother_id].dimentions.outer_height()
        )}
        None => {
            match parent {
                Some(parent_id) => {(
                    components[parent_id].dimentions.content.x + components[parent_id].dimentions.padding.left,
                    components[parent_id].dimentions.content.y + components[parent_id].dimentions.padding.top
                )}
                None => (0.0, 0.0)
            }
        }
    };

    match &node.node_type {
        NodeType::Element { tag, attributes } => {
            if matches!(tag.as_str(), "script" | "style"){
                return None;
            }
            let dimentions = match tag.as_str() {
                "script" | "style" => {
                    return None;
                }
                "body" => {
                    Dimentions {
                        //                     :(          :(
                        content: Rect { x: x + 8.0, y: y + 8.0, width: 100.0, height: 20.0},
                        margin: EdgeSize {
                            left: 8.0,
                            right: 8.0,
                            top: 8.0,
                            bottom: 8.0,
                        },
                        ..Default::default()
                    }
                }
                "h1" => {
                    Dimentions {
                        content: Rect { x: x, y: y, width: 100.0, height: 20.0 },
                        ..Default::default()
                    }
                }
                "p" | "a" => {
                    Dimentions {
                        content: Rect { x: x, y: y, width: 100.0, height: 10.0 },
                        ..Default::default()
                    }
                }
                _ => {
                    Dimentions {
                        content: Rect{ x: x, y: y, width: 100.0, height: 0.0},
                        ..Default::default()
                    }
                }
            };
            components.push(Component {
                dimentions: dimentions,
                text: None,
            });
        }
        NodeType::Text(text) => {
            components.push(Component {
                dimentions: Dimentions {
                    content: Rect { x: x, y: y, width: 100.0, height: 20.0 },
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
        height += components.last().unwrap().dimentions.outer_height();
        last_child = Some(child);
    }

    components[id].dimentions.content.height = height;

    return Some(id);
}