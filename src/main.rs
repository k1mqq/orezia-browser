mod http_client;
mod html_parser;
mod renderer;

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

    let dom = html_parser::parse( response.body);

    println!("{:?}", dom);

    renderer::render();
    Ok(())
}
