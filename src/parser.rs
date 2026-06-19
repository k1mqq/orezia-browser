pub mod html_tokenizer;
pub mod html_tree_constructor;

pub use html_tokenizer::Tokenizer;
pub use html_tree_constructor::TreeConstructor;

#[derive(Debug)]
pub struct Dom {
    nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    node_type: NodeType,
    children: Vec<NodeId>,
    parent: Option<NodeId>
}

type NodeId = usize;

#[derive(Debug)]
pub enum NodeType{
    Document,
    Element{ tag: String, attributes: Vec<(String, String)> },
    Text(String),
    // Comment(String),
}

#[derive(Debug)]
pub enum Token {
    // Doctype {
    //     name: String,
    //     public_id: Option<String>,
    //     system_id: Option<String>,
    //     force_quirks: bool,
    // },
    StartTag {
        name: String,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    },
    // Comment(String),
    Character(char),
    Eof,
}