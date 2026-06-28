pub mod tokenizer;
pub mod tree_constructor;

pub use tokenizer::Tokenizer;
pub use tree_constructor::TreeConstructor;

#[derive(Debug)]
pub struct Dom {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
}

pub type NodeId = usize;

#[derive(Debug)]
pub enum NodeType {
    Document,
    Element {
        tag: String,
        attributes: Vec<(String, String)>,
    },
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

impl Dom {
    pub fn print(&self, node: NodeId, depth: usize) {
        let indent = "  ".repeat(depth);
        match &self.nodes[node].node_type {
            NodeType::Document => {}
            NodeType::Element { tag, attributes } => {
                println!("{}| <{}>", indent, tag);
                for (name, value) in attributes {
                    println!("{}|    {}=\"{}\"", indent, name, value);
                }
                for &child in &self.nodes[node].children {
                    self.print(child, depth + 1);
                }
            }
            NodeType::Text(t) => {
                println!("{}| \"{}\"", indent, t);
            }
        }
    }
}

pub fn parse(input: String) -> Dom {
    let mut tokenizer = Tokenizer::new(&input);
    let mut tree_builder = TreeConstructor::new();

    loop {
        let token = tokenizer.next_token();
        tree_builder.process_token(&token);
        if matches!(token, Token::Eof) {
            break;
        }
    }

    tree_builder.dom
}
