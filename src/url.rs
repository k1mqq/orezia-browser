#[derive(Debug)]
pub struct URL {
    pub scheme: String,
    pub host: String,

    pub port: Option<u16>,
    pub path: String,
}

enum ParserState {
    SchemeStart,
    Scheme,
    NoScheme,
    // special relative of authority state,
    // PathOrRelative,
    Relative,
    RelativeSlash,
    SpecialAuthoritySlashes,
    SpecialAuthorityIgnoreSlashes,
    Authority,
    Host,
    Port,
    // File,
    // FileSlash,
    // FileHost
    PathStart,
    Path,
    // OpaquePath,
    // Query,
    // Fragment,
    // No query and fragment. these are counted as part of path
}

impl URL {
    pub fn parse(text: String) -> Option<URL> {
        let mut pointer = 0;
        let mut state = ParserState::SchemeStart;
        let mut buffer = String::new();
        let mut url = URL{ scheme: "".to_string(), host: "".to_string(), port: None, path:"".to_string() };

        loop {
            let Some(c) = text.chars().nth(pointer) else {
                break;
            };
            // println!("{}", c);
            match state {
                ParserState::SchemeStart => {
                    if c.is_ascii_alphabetic() {
                        buffer.push(c.to_ascii_lowercase());
                        state = ParserState::Scheme;
                    } else {
                        state = ParserState::NoScheme;
                        pointer -= 1;
                    }
                }
                ParserState::Scheme => {
                    if c.is_ascii_alphanumeric() | matches!(c, '+' | '-' | '.') {
                        buffer.push(c.to_ascii_lowercase());
                    } else if matches!(c, ':') {
                        url.scheme = buffer.clone();
                        buffer.clear();
                        state = ParserState::SpecialAuthoritySlashes;
                    } else {
                        return None;
                    }
                }
                ParserState::NoScheme => {
                    if matches!(c, '#') {
                        return None;
                    } else {
                        state = ParserState::Relative;
                        pointer -= 1;
                    }
                }
                ParserState::Relative => {
                    if matches!(c, '/') {
                        state = ParserState::RelativeSlash;
                    } else {
                        // if matches!(c, '?') {
                        //     state = ParserState::Query;
                        // } else if matches!(c, '#') {
                        //     state = ParserState::Fragment;
                        // } else {
                        state = ParserState::Path;
                        pointer -= 1;
                        // }
                    }
                }
                ParserState::RelativeSlash => {
                    if matches!(c, '/' | '\\') {
                        state = ParserState::SpecialAuthoritySlashes;
                    } else {
                        state = ParserState::Path;
                    }
                }
                ParserState::SpecialAuthoritySlashes => {
                    let Some(next) = text.chars().nth(pointer + 1) else {
                        // it has to have fallback but i dont care
                        return None;
                    };

                    if matches!(c, '/') && matches!(next, '/') {
                        pointer += 1;
                        state = ParserState::SpecialAuthorityIgnoreSlashes;
                    } else {
                        // it has to have fallback but i dont care
                        return None;
                    }
                }
                ParserState::SpecialAuthorityIgnoreSlashes => {
                    if matches!(c, '/' | '\\') {
                        return None;
                    } else {
                        pointer -= 1;
                        state = ParserState::Authority;
                    }
                }
                ParserState::Authority => {
                    if matches!(c, '@') {
                        println!("I don't support URL Auth!");
                        return None;
                    } else if matches!(c, '/' | '\\' | '?' | '#') {
                        pointer -= buffer.len() + 1;
                        buffer.clear();
                        state = ParserState::Host;
                    } else {
                        buffer.push(c);
                    }
                }
                ParserState::Host => {
                    if matches!(c, ':') {
                        if buffer.is_empty() {
                            return None;
                        }

                        url.host = buffer.clone();
                        buffer.clear();

                        state = ParserState::Port;
                    } else if matches!(c, '/' | '\\' | '?' | '#') {
                        if buffer.is_empty() {
                            return None;
                        }

                        pointer -= 1;
                        url.host = buffer.clone();
                        buffer.clear();

                        state = ParserState::PathStart;
                    } else {
                        buffer.push(c);
                    }
                }
                ParserState::Port => {
                    if c.is_numeric() {
                        buffer.push(c);
                    } else if matches!(c, '/' | '\\' | '?' | '#') {
                        let Ok(port) = u16::from_str_radix(buffer.as_str(), 10) else {
                            return None;
                        };
                        url.port = Some(port);
                    } else {
                        return None;
                    }
                }
                ParserState::PathStart => {
                    state = ParserState::Path;
                    if !matches!(c, '/' | '\\') {
                        pointer -= 1;
                    }
                    continue;
                }
                ParserState::Path => {
                    println!("{}", c);
                    if matches!(c, '/' | '\\') {
                        // ok?
                        url.path.push_str(buffer.clone().as_str());
                        buffer.clear();
                    } else if is_url_code(c) {
                        buffer.push(c);
                    }
                }
            }
            pointer += 1;
        }
        // :<
        url.path.push_str(buffer.clone().as_str());
        Some(url)
    }
}

fn is_url_code(c: char) -> bool {
    c.is_ascii_alphanumeric() | matches!(c, '!' | '$' | '&' | '\'' | '*' | '+' | ',' | '-' | '.' | '/' | ':' | ';' | '=' | '?' | '@' | '_' | '~')
}