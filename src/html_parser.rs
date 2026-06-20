pub mod tokenizer;
pub mod tree_constructor;

pub use tokenizer::Tokenizer;
pub use tree_constructor::TreeConstructor;

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

pub fn parse(input: String) -> Dom{
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