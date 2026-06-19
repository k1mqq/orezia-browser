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
enum NodeType{
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

enum TokenizerState {
    Data,
    TagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    EndTagOpen,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnQuoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
}

enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    AfterBody,
}

pub struct Tokenizer {
    input: Vec<char>,
    pos: usize,
    current_tag_name: String,
    // does not look good but enough
    is_current_tag_open: bool,
    current_tag_self_closing: bool,
    current_attribute_name: String,
    current_attribute_value: String,
    current_attributes: Vec<(String, String)>,
    state: TokenizerState,
}

pub struct TreeConstructor {
    pub dom: Dom,
    mode: InsertionMode,
    open_elements: Vec<NodeId>,
}

impl Tokenizer {
    pub fn new(input: &String) -> Self {
        let input_chars = input.chars().collect();
        Tokenizer {
            input: input_chars,
            pos: 0,
            current_tag_name: "".to_string(),
            is_current_tag_open: false,
            current_tag_self_closing: false,
            current_attribute_name: "".to_string(),
            current_attribute_value: "".to_string(),
            current_attributes: Vec::new(),
            state: TokenizerState::Data, 
        }
    }
    pub fn next_token(&mut self) -> Token {
        loop {
            let ch = self.current_char();
            match self.state {
                TokenizerState::Data => match ch {
                    Some('<') => {
                        self.consume();
                        self.state = TokenizerState::TagOpen;
                    }
                    Some(c) => {
                        self.consume();
                        return Token::Character(c)
                    },
                    None => {
                        self.consume();
                        return Token::Eof;
                    }
                }
                TokenizerState::TagOpen => {
                    match ch {
                        Some('/') => {
                            self.consume();
                            self.state = TokenizerState::EndTagOpen;
                        }
                        Some(c) if c.is_ascii_alphabetic() => {
                            // create a new star tag token
                            self.current_tag_name = "".to_string();
                            self.is_current_tag_open = true;
                            self.state = TokenizerState::TagName;
                        }
                        Some(_) => {
                            // this is error
                            self.state = TokenizerState::Data;
                            return Token::Character('<');
                        }
                        None => {
                            // this is also error
                            self.consume();
                            // tokens.push(Token::Character('<'));
                            // tokens.push(Token::Eof);
                            // cannot emit Character Token :thinking:
                            return Token::Eof;
                        }
                    }
                }
                TokenizerState::TagName => {
                    match ch {
                        // no escape for FF in rust
                        Some('\t') | Some('\n') | Some(' ') => {
                            self.consume();
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        Some('/') => {
                            self.consume();
                            self.state = TokenizerState::SelfClosingStartTag;
                        }
                        Some('>') => {
                            self.consume();
                            self.state = TokenizerState::Data;
                            if self.is_current_tag_open {
                                // is clone ok? maybe yes.
                                return Token::StartTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };

                            }else {
                                return Token::EndTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };
                            }
                        }
                        Some(c) => {
                            self.consume();
                            self.current_tag_name.push(c.to_ascii_lowercase());
                        }
                        None => {
                            self.consume();
                            // ERROR
                            return Token::Eof;
                        }
                    }
                }
                TokenizerState::BeforeAttributeName => {
                    match ch {
                        Some('\t') | Some('\n') | Some(' ') => {
                            self.consume();
                        }
                        Some('/') | Some('>') | None => {
                            self.state = TokenizerState::AfterAttributeName;
                        }
                        Some(_) => {
                            if !self.current_attribute_name.is_empty() {
                                self.current_attributes.push((
                                    self.current_attribute_name.clone(),
                                    self.current_attribute_value.clone()
                                ));
                            }
                            self.current_attribute_name = "".to_string();
                            self.current_attribute_value = "".to_string();
                            self.state = TokenizerState::AttributeName;
                        }
                    }

                }
                TokenizerState::AttributeName => {
                    match ch {
                        Some('\t') | Some('\n') | Some(' ') | Some('/') | None => {
                            self.state = TokenizerState::AfterAttributeName;
                        }
                        Some('=') => {
                            self.consume();
                            self.state = TokenizerState::BeforeAttributeValue;
                        }
                        Some(c) => {
                            self.consume();
                            self.current_attribute_name.push(c.to_ascii_lowercase())
                        }
                    }
                }
                TokenizerState::AfterAttributeName => {
                    match ch {
                        Some('\t') | Some('\n') | Some(' ') => {
                            self.consume();
                        }
                        Some('/') => {
                            self.consume();
                            self.state = TokenizerState::SelfClosingStartTag;
                        }
                        Some('=') => {
                            self.consume();
                            self.state = TokenizerState::BeforeAttributeValue;
                        }
                        Some('>') => {
                            self.consume();
                            self.state = TokenizerState::Data;

                            // last attribute is not pushed to this point
                            self.current_attributes.push((
                                self.current_attribute_name.clone(),
                                self.current_attribute_value.clone()
                            ));
                            if self.is_current_tag_open {
                                return Token::StartTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing,
                                };
                            } else {
                                return Token::EndTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing,
                                };
                            }
                        }
                        Some(_) => {
                            if !self.current_attribute_name.is_empty() {
                                self.current_attributes.push((
                                    self.current_attribute_name.clone(),
                                    self.current_attribute_value.clone()
                                ));
                            }
                            self.current_attribute_name = "".to_string();
                            self.current_attribute_value = "".to_string();
                            self.state = TokenizerState::AttributeName;
                        }
                        None => {
                            // error
                            self.consume();
                            return Token::Eof;
                        }
                    }
                }
                TokenizerState::EndTagOpen => {
                    match ch {
                        Some(ascii_alpha) if ascii_alpha.is_ascii_alphabetic() => {
                            self.current_tag_name = "".to_string();
                            self.is_current_tag_open = false;
                            self.state = TokenizerState::TagName;
                        }
                        Some('>') => {
                            // error
                            self.consume();
                            self.state = TokenizerState::Data;
                        }
                        Some(_) => {
                            // error
                        }
                        None => {
                            // error
                            self.consume();
                            // tokens.push(Token::Character('<'));
                            // tokens.push(Token::Character('/'));
                            // tokens.push(Token::Eof);
                            // ignored 2 token :sob:
                            return Token::Eof;
                        }
                    }
                }
                TokenizerState::BeforeAttributeValue => {
                    match ch {
                        Some('\t') | Some('\n') | Some(' ') => {
                            self.consume();
                        }
                        Some('"') => {
                            self.consume();
                            self.state = TokenizerState::AttributeValueDoubleQuoted;
                        }
                        Some('\'') => {
                            self.consume();
                            self.state = TokenizerState::AttributeValueSingleQuoted;
                        }
                        Some('>') => {
                            // ERROR
                            self.consume();
                            self.state = TokenizerState::Data;
                            if self.is_current_tag_open {
                                return Token::StartTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };
                            } else {
                                return Token::EndTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };
                            }
                        }
                        Some(_) | None => {
                            self.state = TokenizerState::AttributeValueUnQuoted;
                        }
                    }
                }
                TokenizerState::AttributeValueDoubleQuoted => {
                    match ch {
                        Some('"') => {
                            self.consume();
                            self.state = TokenizerState::AfterAttributeValueQuoted;
                        }
                        Some(c) => {
                            self.consume();
                            self.current_attribute_value.push(c);
                        }
                        None => {
                            // ERROr
                            self.consume();
                            return Token::Eof;
                        }
                    }
                }
                TokenizerState::AttributeValueSingleQuoted => {
                    match ch {
                        Some('\'') => {
                            self.consume();
                            self.state = TokenizerState::AfterAttributeValueQuoted;
                        }
                        Some(c) => {
                            self.consume();
                            self.current_attribute_value.push(c);
                        }
                        None => {
                            // ERROr
                            self.consume();
                            return Token::Eof;
                        }
                    }
                }
                TokenizerState::AttributeValueUnQuoted => {
                    match ch {
                        Some('\t') | Some('\n') | Some(' ') => {
                            self.consume();
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        Some('>') => {
                            self.consume();
                            self.state = TokenizerState::Data;
                            if self.is_current_tag_open {
                                return Token::StartTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };
                            } else {
                                return Token::EndTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };
                            }
                        }
                        Some(c) => {
                            self.consume();
                            self.current_attribute_value.push(c);
                        }
                        None => {
                            // Errorr
                            self.consume();
                            return Token::Eof;
                        }
                    }
                }
                TokenizerState::AfterAttributeValueQuoted => {
                    match ch {
                        Some('\t') | Some('\n') | Some(' ') => {
                            self.consume();
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        Some('/') => {
                            self.consume();
                            self.state = TokenizerState::SelfClosingStartTag;
                        }
                        Some('>') => {
                            self.consume();
                            self.state = TokenizerState::Data;
                            if self.is_current_tag_open {
                                return Token::StartTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };
                            } else {
                                return Token::EndTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };
                            }
                        }
                        Some(_) => {
                            // ERROR
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        None => {
                            // Errorr
                            self.consume();
                            return Token::Eof;
                        }
                    }
                }
                TokenizerState::SelfClosingStartTag => {
                    match ch {
                        Some('>') => {
                            self.consume();
                            self.current_tag_self_closing = true;
                            self.state = TokenizerState::Data;
                            if self.is_current_tag_open {
                                return Token::StartTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };
                            } else {
                                return Token::EndTag {
                                    name: self.current_tag_name.clone(),
                                    attributes: self.current_attributes.clone(),
                                    self_closing: self.current_tag_self_closing
                                };
                            }
                        }
                        Some(_) => {
                            // error
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        None => {
                            // Errorr
                            self.consume();
                            return Token::Eof;
                        }
                    }
                }
            }
        }
    }

    fn consume(&mut self) {
        self.pos += 1;
    }
    fn current_char(&mut self) -> Option<char>{
        self.input.get(self.pos).copied()
    }
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