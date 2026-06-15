#[derive(Debug)]
pub enum Token {
    Doctype {
        name: String,
        public_id: Option<String>,
        system_id: Option<String>,
        force_quirks: bool,
    },
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
    Comment(String),
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
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
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
                        tokens.push(Token::Character(c));
                    },
                    None => {
                        self.consume();
                        tokens.push(Token::Eof);
                        break;
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
                            tokens.push(Token::Character('<'));
                            self.state = TokenizerState::Data;
                        }
                        None => {
                            // this is also error
                            self.consume();
                            tokens.push(Token::Character('<'));
                            tokens.push(Token::Eof);
                            break;
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
                            if self.is_current_tag_open {
                                // is clone ok? maybe yes.
                                tokens.push(Token::StartTag { name: self.current_tag_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing });
                            }else {
                                tokens.push(Token::EndTag { name: self.current_attribute_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing });
                            }
                            self.state = TokenizerState::Data;
                        }
                        Some(c) => {
                            self.consume();
                            self.current_tag_name.push(c.to_ascii_lowercase());
                        }
                        None => {
                            self.consume();
                            // ERROR
                            tokens.push(Token::Eof);
                            break;
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
                                tokens.push(Token::StartTag {
                                    name: self.current_tag_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing,
                                });
                            } else {
                                tokens.push(Token::EndTag {
                                    name: self.current_tag_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing,
                                });
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
                            tokens.push(Token::Eof);
                            break;
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
                            tokens.push(Token::Character('<'));
                            tokens.push(Token::Character('/'));
                            tokens.push(Token::Eof);
                            break;
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
                                tokens.push(Token::StartTag { name: self.current_attribute_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing })
                            } else {
                                tokens.push(Token::EndTag { name: self.current_attribute_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing })
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
                            tokens.push(Token::Eof);
                            break;
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
                            tokens.push(Token::Eof);
                            break;
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
                                tokens.push(Token::StartTag { name: self.current_attribute_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing })
                            } else {
                                tokens.push(Token::EndTag { name: self.current_attribute_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing })
                            }
                        }
                        Some(c) => {
                            self.consume();
                            self.current_attribute_value.push(c);
                        }
                        None => {
                            // Errorr
                            self.consume();
                            tokens.push(Token::Eof);
                            break;
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
                                tokens.push(Token::StartTag { name: self.current_attribute_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing })
                            } else {
                                tokens.push(Token::EndTag { name: self.current_attribute_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing })
                            }
                        }
                        Some(_) => {
                            // ERROR
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        None => {
                            // Errorr
                            self.consume();
                            tokens.push(Token::Eof);
                            break;
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
                                tokens.push(Token::StartTag { name: self.current_attribute_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing })
                            } else {
                                tokens.push(Token::EndTag { name: self.current_attribute_name.clone(), attributes: self.current_attributes.clone(), self_closing: self.current_tag_self_closing })
                            }
                        }
                        Some(_) => {
                            // error
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        None => {
                            // Errorr
                            self.consume();
                            tokens.push(Token::Eof);
                            break;
                        }
                    }
                }
            }
        }

        tokens
    }

    fn consume(&mut self) {
        self.pos += 1;
    }
    fn current_char(&mut self) -> Option<char>{
        self.input.get(self.pos).copied()
    }
}