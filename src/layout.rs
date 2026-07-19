use fontdue::{
    Font,
    layout::{CoordinateSystem, TextStyle},
};

use crate::html_parser::{Dom, Node, NodeId, NodeType};

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
    Inline,
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

        next_component(&mut components, dom, body_id, None, None, &context);

        Self { components }
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
        self.inner_width()
            + self.border.left
            + self.border.right
            + self.margin.left
            + self.margin.right
    }

    fn outer_height(&self) -> f32 {
        self.inner_height()
            + self.border.top
            + self.border.bottom
            + self.margin.top
            + self.margin.bottom
    }
}

fn next_component(
    components: &mut Vec<Component>,
    dom: &Dom,
    node_id: NodeId,
    parent_id: Option<ComponentId>,
    brother_id: Option<ComponentId>,
    context: &LayoutContext,
) -> Option<ComponentId> {
    // println!("{}", node_id);
    let node = &dom.nodes[node_id];
    let id = components.len();

    let brother = if let Some(id) = brother_id {
        components.get(id)
    } else {
        None
    };

    let parent = if let Some(id) = parent_id {
        components.get(id)
    } else {
        None
    };

    let margin = calc_margin(node);
    let text = node.get_text();
    let box_type = node.box_type();

    let (x, y) = calc_pos(node, brother, parent);

    let width = calc_width(node, parent, context);

    let height = calc_height(node, context);

    components.push(Component {
        dimentions: Dimentions {
            content: Rect {
                x: x + margin.left,
                y: y + margin.top,
                width: width,
                height: height,
            },
            margin: margin,
            ..Default::default()
        },
        text: text.cloned(),
        box_type: box_type.clone(),
    });

    let mut last_child = None;
    let mut height = components[id].dimentions.content.height;
    let mut width = components[id].dimentions.content.width;
    for child_node in &node.children {
        let Some(child) =
            next_component(components, dom, *child_node, Some(id), last_child, context)
        else {
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

fn calc_margin(node: &Node) -> EdgeSize {
    match &node.node_type {
        NodeType::Element { tag, attributes: _ } => {
            if matches!(tag.as_str(), "script" | "style") {
                return EdgeSize::default();
            }
            match tag.as_str() {
                "script" | "style" => {
                    return EdgeSize::default();
                }
                "body" => {
                    return EdgeSize {
                        left: 8.0,
                        right: 8.0,
                        top: 8.0,
                        bottom: 8.0,
                    };
                }
                _ => {
                    return EdgeSize::default();
                }
            };
        }
        _ => {
            return EdgeSize::default();
        }
    }
}

fn calc_pos(node: &Node, brother: Option<&Component>, parent: Option<&Component>) -> (f32, f32) {
    match node.box_type() {
        BoxType::Block => match brother {
            Some(brother) => (
                brother.dimentions.content.x,
                brother.dimentions.content.y + brother.dimentions.outer_height(),
            ),
            None => match parent {
                Some(parent) => (
                    parent.dimentions.content.x + parent.dimentions.padding.left,
                    parent.dimentions.content.y + parent.dimentions.padding.top,
                ),
                None => (0.0, 0.0),
            },
        },
        BoxType::Inline => {
            match brother {
                Some(brother) => {
                    (
                        // imperfect
                        brother.dimentions.content.x + brother.dimentions.outer_width(),
                        brother.dimentions.content.y,
                    )
                }
                None => match parent {
                    Some(parent) => (
                        parent.dimentions.content.x + parent.dimentions.padding.left,
                        parent.dimentions.content.y + parent.dimentions.padding.top,
                    ),
                    None => (0.0, 0.0),
                },
            }
        }
    }
}

fn calc_width(node: &Node, parent: Option<&Component>, context: &LayoutContext) -> f32 {
    match node.box_type() {
        BoxType::Block => match parent {
            Some(parent) => parent.dimentions.content.width,
            None => context.window_width as f32,
        },
        BoxType::Inline => match &node.get_text() {
            Some(text) => text
                .chars()
                .map(|c| {
                    let metrics = context.font.metrics(c, 20.0);
                    metrics.advance_width.ceil()
                })
                .sum(),
            None => 0.0,
        },
    }
}

fn calc_height(node: &Node, context: &LayoutContext) -> f32 {
    match node.get_text() {
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
        None => 0.0,
    }
}
