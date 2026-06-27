use fontdue::{Font, layout::{CoordinateSystem, TextStyle}};

use crate::html_parser::{Dom, NodeId, NodeType};

pub struct Layout {
    pub components: Vec<Component>,
}

pub struct LayoutContext<'a> {
    pub font: &'a Font,
    pub window_height: u32,
    pub window_width: u32,
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

impl<'a> Layout {
    pub fn build(dom: &Dom, context: LayoutContext<'a>) -> Layout {
        let mut components = Vec::new();

        let body_id = dom.nodes.iter().position(|node|
            if let NodeType::Element { tag, attributes: _ } = &node.node_type {
                tag == "body"
            } else {
                false
            }
        ).expect("no body element?");

        next_component(&mut components, dom, body_id, None, None, &context);

        Self {
            components,
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

fn next_component(components: &mut Vec<Component>, dom: &Dom, node_id: NodeId, parent: Option<ComponentId>, brother: Option<ComponentId>, context: &LayoutContext) -> Option<ComponentId> {
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

    let width = match parent {
        Some(parent_id) => {
            components[parent_id].dimentions.content.width
        }
        None => {
            context.window_width as f32
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
                        content: Rect { x: x + 8.0, y: y + 8.0, width: width - 8.0, height: 20.0},
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
                        content: Rect { x: x, y: y, width: width, height: 20.0 },
                        ..Default::default()
                    }
                }
                "p" | "a" | "span"=> {
                    Dimentions {
                        content: Rect { x: x, y: y, width: width, height: 10.0 },
                        ..Default::default()
                    }
                }
                _ => {
                    Dimentions {
                        content: Rect{ x: x, y: y, width: width, height: 0.0},
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
            // recreate every time for no reason
            // fix later
            let mut font_layout = fontdue::layout::Layout::new(CoordinateSystem::PositiveYDown);
            let font_layout_settings = fontdue::layout::LayoutSettings {
                max_width: Some(width),
                ..Default::default()
            };
            font_layout.reset(&font_layout_settings);

            font_layout.append(&[context.font], &TextStyle::new(&text, 20.0, 0));

            components.push(Component {
                dimentions: Dimentions {
                    content: Rect { x: x, y: y, width: width, height: font_layout.height() },
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
    let mut height = components[id].dimentions.content.height;
    for child_node in &node.children {
        let Some(child) = next_component(components, dom, *child_node, Some(id), last_child, context) else {
            continue;
        };
        height += components[child].dimentions.outer_height();
        last_child = Some(child);
    }

    components[id].dimentions.content.height = height;

    return Some(id);
}