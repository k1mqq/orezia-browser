use crate::html_parser::{Node, NodeType};

// stole https://github.com/mbrubeck/robinson/blob/master/src/css.rs
// because it is too good
#[derive(Clone, Debug)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Debug)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(Clone, Debug)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

#[derive(Clone, Debug)]
pub enum Unit {
    Px,
    Pt,
    // Percent,
}

#[derive(Clone, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub struct Parser {
    input: String,
    pos: usize,
}

pub type Specificity = (usize, usize, usize);

impl Selector {
    // what is this?
    pub fn specificity(&self) -> Specificity {
        // http://www.w3.org/TR/selectors/#specificity
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }

    pub fn matches(&self, node: &Node) -> bool {
        match self {
            Self::Simple(s) => s.matches(node),
        }
    }
}

impl SimpleSelector {
    pub fn matches(&self, node: &Node) -> bool {
        match &node.node_type {
            NodeType::Element { tag, attributes } => {
                if let Some(selector_tag) = &self.tag_name {
                    if selector_tag != tag {
                        return false;
                    }
                }
                if let Some(selector_id) = &self.id {
                    let id_matches = attributes
                        .iter()
                        .any(|(k, v)| k == "id" && v == selector_id);
                    if !id_matches {
                        return false;
                    }
                }

                if !self.class.is_empty() {
                    let element_classes: Vec<&str> = attributes
                        .iter()
                        .filter(|(k, _)| k == "class")
                        .flat_map(|(_, v)| v.split_whitespace())
                        .collect();
                    let all_present = self
                        .class
                        .iter()
                        .all(|c| element_classes.iter().any(|ec| ec == c));
                    if !all_present {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }
}

impl Parser {
    pub fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();

        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            }
        }
        rules
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = match self.parse_selectors() {
            Some(selectors) if !selectors.is_empty() => selectors,
            Some(_) => return None,
            None => {
                self.skip_rule();
                return None;
            }
        };
        Some(Rule {
            selectors: selectors,
            declarations: self.parse_declarations(),
        })
    }

    fn parse_selectors(&mut self) -> Option<Vec<Selector>> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                Some(',') => {
                    self.consume_char();
                    self.consume_whitespace();
                }

                Some('{') => break,
                Some('[') | Some(':') => {
                    self.skip_rule();
                    return Some(Vec::new());
                }
                _ => return None,
            }
        }
        selectors.sort_by_key(|s| s.specificity());
        Some(selectors)
    }

    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };
        while let Some(c) = self.next_char() {
            match c {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }

                _ => break,
            }
        }
        selector
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        if !self.expect_char('{') {
            return Vec::new();
        }
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            match self.next_char() {
                None => break,
                Some('}') => {
                    self.consume_char();
                    break;
                }
                Some(_) => match self.parse_declaration() {
                    Some(declaration) => declarations.push(declaration),
                    None => self.skip_declaration(),
                },
            }
        }
        declarations
    }

    fn parse_declaration(&mut self) -> Option<Declaration> {
        let name = self.parse_identifier();
        if name.is_empty() {
            return None;
        }
        self.consume_whitespace();
        if !self.expect_char(':') {
            return None;
        }
        self.consume_whitespace();
        let value = self.parse_value()?;
        self.consume_whitespace();
        if !self.expect_char(';') {
            return None;
        }

        Some(Declaration { name, value })
    }

    fn skip_declaration(&mut self) {
        while let Some(c) = self.next_char() {
            if c == '}' {
                break;
            }
            self.pos += c.len_utf8();
            if c == ';' {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Option<Value> {
        match self.next_char()? {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => {
                let keyword = self.parse_identifier();
                if keyword.is_empty() {
                    None
                } else {
                    Some(Value::Keyword(keyword))
                }
            }
        }
    }

    fn parse_length(&mut self) -> Option<Value> {
        let value = self.parse_float()?;
        let unit = self.parse_unit()?;
        Some(Value::Length(value, unit))
    }

    fn parse_float(&mut self) -> Option<f32> {
        self.consume_while(|c| matches!(c, '0'..='9' | '.'))
            .parse()
            .ok()
    }

    fn parse_unit(&mut self) -> Option<Unit> {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Some(Unit::Px),
            "pt" => Some(Unit::Pt),
            _ => None,
        }
    }

    fn parse_color(&mut self) -> Option<Value> {
        if !self.expect_char('#') {
            return None;
        }
        Some(Value::ColorValue(Color {
            r: self.parse_hex_pair()?,
            g: self.parse_hex_pair()?,
            b: self.parse_hex_pair()?,
            a: 255,
        }))
    }

    fn parse_hex_pair(&mut self) -> Option<u8> {
        let s = &self.input[self.pos..self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).ok()
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    // うおw
    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            if !test(c) {
                break;
            }
            self.pos += c.len_utf8();
            result.push(c);
        }
        result
    }

    fn consume_char(&mut self) -> Option<char> {
        let c = self.next_char()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn expect_char(&mut self, c: char) -> bool {
        if self.next_char() == Some(c) {
            self.consume_char();
            true
        } else {
            false
        }
    }

    fn next_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn skip_rule(&mut self) {
        while let Some(c) = self.next_char() {
            self.pos += c.len_utf8();
            if c == '}' {
                break;
            }
        }
    }
}

fn valid_identifier_char(c: char) -> bool {
    // TODO: Include U+00A0 and higher.
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}

pub fn parse(input: &str) -> StyleSheet {
    let mut parser = Parser {
        input: input.to_string(),
        pos: 0,
    };
    StyleSheet {
        rules: parser.parse_rules(),
    }
}
