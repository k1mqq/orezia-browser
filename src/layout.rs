use std::default;

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
    pub box_type: BoxType,
}

#[derive(Clone)]
pub enum BoxType {
    Block,
    Inline
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

    let (margin, text, box_type) = match &node.node_type {
        NodeType::Element { tag, attributes } => {
            let mut margin = EdgeSize::default();
            let mut box_type = BoxType::Block;
            if matches!(tag.as_str(), "script" | "style"){
                return None;
            }
            let dimentions = match tag.as_str() {
                "script" | "style" => {
                    return None;
                }
                "body" => {
                    margin = EdgeSize { left: 8.0, right: 8.0, top: 8.0, bottom: 8.0 };
                }
                "a" | "span"=> {
                    box_type = BoxType::Inline;
                }
                _ => {
                }
            };
            (margin, None, box_type)
        }
        NodeType::Text(text) => {
            (EdgeSize::default(), Some(text.to_string()), BoxType::Inline)
        }
        _ => {
            return None;
        }
    };

    let (x, y) = match box_type {
        BoxType::Block => {
            match brother {
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
            }
        }
        BoxType::Inline => {
            match brother {
                Some(brother_id) => {(
                    // imperfect
                    components[brother_id].dimentions.content.x + components[brother_id].dimentions.outer_width(),
                    components[brother_id].dimentions.content.y
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
            }
        }
    };

    let width = match box_type {
        BoxType::Block => {
            match parent {
                Some(parent_id) => {
                    components[parent_id].dimentions.content.width
                }
                None => {
                    context.window_width as f32
                }
            }
        }
        BoxType::Inline => {
            match &text {
                Some(text) => {
                    text.chars().map(|c| {
                        let metrics = context.font.metrics(c, 20.0);
                        metrics.advance_width.ceil()
                    }).sum()
                }
                None => 0.0
            }
        }
    };

    let height = match &text {
        Some(text) => {
            let mut font_layout = fontdue::layout::Layout::new(CoordinateSystem::PositiveYDown);
            let font_layout_settings = fontdue::layout::LayoutSettings {
                // max_width: Some(width),
                ..Default::default()
            };
            font_layout.reset(&font_layout_settings);

            font_layout.append(&[context.font], &TextStyle::new(&text, 20.0, 0));
            font_layout.height()
        }
        None => 0.0
    };

    components.push(Component {
        dimentions: Dimentions {
            content: Rect{ x: x + margin.left, y: y + margin.top, width: width, height: height },
            margin: margin,
            ..Default::default()
        },
        text: text,
        box_type: box_type.clone()
    });

    let mut last_child = None;
    let mut height = components[id].dimentions.content.height;
    let mut width = components[id].dimentions.content.width;
    for child_node in &node.children {
        let Some(child) = next_component(components, dom, *child_node, Some(id), last_child, context) else {
            continue;
        };
        match box_type {
            BoxType::Block => {
                height += components[child].dimentions.outer_height();
                width = width.max(components[child].dimentions.outer_width());
            }
            BoxType::Inline => {
                width += components[child].dimentions.outer_width();
                height = height.max(components[child].dimentions.outer_height());
            }
        }
        last_child = Some(child);
    }

    components[id].dimentions.content.width = width;
    components[id].dimentions.content.height = height;

    return Some(id);
}