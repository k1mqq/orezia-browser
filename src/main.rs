mod http_client;
mod parser;

use std::{collections::HashMap, error::Error};

fn main()  -> Result<(), Box<dyn Error>> {
    let mut headers = HashMap::new();
    headers.insert("Host".to_string(), "example.com".to_string());
    headers.insert("User-Agent".to_string(), "orezia-browser/0.0".to_string());
    let response = http_client::Request::new(
        String::from("example.com:80"),
        String::from("/"),
        http_client::Method::GET,
        headers,
        String::from("")).send()?;
    println!("{}", &response.status);
    println!("{}", &response.body);

    let mut tokenizer = parser::Tokenizer::new(&response.body);
    let mut tree_builder = parser::TreeConstructor::new();

    loop {
        let token = tokenizer.next_token();
        tree_builder.process_token(&token);
        if matches!(token, parser::Token::Eof) {
            break;
        }
    }

    println!("{:?}", &tree_builder.dom);
    Ok(())
}
