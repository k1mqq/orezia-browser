use crate::html_parser::*;

enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    AfterBody,
}

pub struct TreeConstructor {
    pub dom: Dom,
    mode: InsertionMode,
    open_elements: Vec<NodeId>,
}

impl TreeConstructor {
    pub fn new() -> Self{
        TreeConstructor {
            dom: Dom{ nodes: vec![Node{node_type:NodeType::Document,children:Vec::new(), parent: None }] },
            mode: InsertionMode::Initial,
            open_elements: Vec::new(),
        }
    }

    pub fn process_token(&mut self, token: &Token) {
        loop {
            // let reprocess =  match self.mode {
            //     InsertionMode::Initial => self.handle_initial(token),
            //     InsertionMode::BeforeHtml => self.handle_before_html(token),
            //     InsertionMode::BeforeHead => self.handle_before_head(token),
            //     InsertionMode::InHead => self.handle_in_head(token),
            //     InsertionMode::AfterHead => self.handle_after_head(token),
            //     InsertionMode::InBody => self.handle_in_body(token),
            //     InsertionMode::AfterBody => self.handle_after_body(token),
            // };
            let reprocess = self.handle_in_body(token);
            if !reprocess {
                break;
            }
        }
    }
    // fn handle_initial(&mut self, _token: &Token) -> bool {
    //     // process DOCTYPE
    //     // do nothing because no DOCTYPE token for now
    //     self.mode = InsertionMode::BeforeHtml;
    //     true
    // }
    // fn handle_before_html(&mut self, token: &Token) -> bool {
    //     match token {
    //         Token::Character(c) if matches!(c, '\t' | '\n' | ' ') => {
    //             return false;
    //         }
    //         Token::StartTag { name, attributes, self_closing } if name == "html" => {
    //             let element = self.create_element(name.clone(), attributes.clone());
    //             self.set_relation(element, 0);
    //             self.open_elements.push(element);
    //             return false;
    //         }
    //         Token::EndTag { name, attributes, self_closing } if !matches!(name.as_str(), "head" | "body" | "br") => {
    //             // error
    //             return false;
    //         }
    //         _ => {
    //             // according to the standard, this process is different from html start tag in some ways.
    //             // but I ignore that.
    //             let element = self.create_element("html".to_string(), Vec::new());
    //             self.set_relation(element, 0);
    //             self.open_elements.push(element);
    //             return false;
    //         }
    //     }
    // }
    // fn handle_before_head(&mut self, token: &Token) -> bool {
    //     match token {
    //         Token::Character(c) if matches!(c, '\t' | '\n' | ' ') => {
    //             return false;
    //         }
    //         Token::StartTag { name, attributes, self_closing } if name == "html" => {
    //             self.mode = InsertionMode::InBody;
    //             return true;
    //         }
    //         Token::StartTag { name, attributes, self_closing } if name == "head" => {
    //             let element = self.create_element(name.clone(), attributes.clone());
    //             self.open_elements.push(element);
    //             self.insert_element(element);
    //             return false;
    //         }
    //         Token::EndTag { name, attributes, self_closing } if !matches!(name.as_str(), "head" | "body" | "html" | "br") => {
    //             // error
    //             return false;
    //         }
    //         _ => {
    //             let element = self.create_element("head".to_string(), Vec::new());
    //             self.open_elements.push(element);
    //             self.insert_element(element);
    //             self.mode = InsertionMode::InHead;
    //             return true;
    //         }
    //     }
    // }
    // fn handle_in_head(&mut self, token: &Token) -> bool {
    //     // match token {
    //     //     Token::Character(c) if matches!(c, '\t' | '\n' | ' ') => {
    //     //         self.insert_character(c);
    //     //         return false;
    //     //     }
    //     // }
    // }
    // fn handle_after_head(&mut self, token: &Token) -> bool {
        
    // }
    fn handle_in_body(&mut self, token: &Token) -> bool {
        match token {
            Token::Character(character) => {
                self.insert_character(*character);
                return false;
            }
            Token::StartTag { name, attributes, self_closing } => {
                let element = self.create_element(name.clone(), attributes.clone());
                self.open_elements.push(element);
                self.insert_element(element);
                return false;
            }
            Token::EndTag { name, attributes, self_closing } => {
                for node_id in self.open_elements.iter().rev() {
                    if let NodeType::Element { tag, attributes}  = &self.dom.nodes[*node_id].node_type {
                        if tag == name {
                            self.open_elements.truncate(*node_id);
                            return false;
                        }
                    }
                }
                return false;
            }
            Token::Eof => {
                return false;
            },
        }
    }
    // fn handle_after_body(&mut self, token: &Token) -> bool {

    // }

    fn create_element(&mut self, name: String, attributes: Vec<(String, String)>) -> usize {
        let id = self.dom.nodes.len();
        let element = Node{
            node_type: NodeType::Element { tag: name, attributes: attributes },
            children: Vec::new(),
            parent: None,
        };

        self.dom.nodes.push(element);
        id
    }

    fn insert_element(&mut self, element_id: NodeId) {
        let current_node_id = *self.open_elements.last().expect("open_element is empty :(");
        match self.dom.nodes.get_mut(element_id) {
            Some(element) => {
                element.parent = Some(current_node_id);
            }
            None => {
                println!("element not found");
            }
        }
        // it must be safe
        self.dom.nodes.get_mut(current_node_id).unwrap().children.push(element_id);
    }

    fn insert_character(&mut self, character:char) {
        let current_node_id = *self.open_elements.last().unwrap_or(&0);
        // Document node dont have Text node
        // or empty node
        if current_node_id == 0 {
            return;
        }
        
        if let Some(&last_child) = self.dom.nodes[current_node_id].children.last() {
            // these are same
            // if let NodeType::Text(ref mut s) = self.dom.nodes[last_child].node_type {
            if let NodeType::Text(s) = &mut self.dom.nodes[last_child].node_type {
                s.push(character);
                return;
            }
        }

        let id = self.dom.nodes.len();
        let text_node = Node {
            node_type: NodeType::Text(character.to_string()),
            children: Vec::new(),
            parent: None,
        };
        self.dom.nodes.push(text_node);
        self.set_relation(id, current_node_id);
    }

    fn set_relation(&mut self, child: NodeId, parent: NodeId) {
        match self.dom.nodes.get_mut(child) {
            Some(child_node) => {
                child_node.parent = Some(parent);
            }
            None => {
                println!("child not found (in set_relation)");
            }
        }
        match self.dom.nodes.get_mut(parent) {
            Some(parent_node) => {
                parent_node.children.push(child);
            }
            None => {
                println!("parent not found (in set_relation)");
            }
        }
    }
}