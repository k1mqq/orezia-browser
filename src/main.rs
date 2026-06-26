mod http_client;
mod html_parser;
mod renderer;
mod layout;

use std::{collections::HashMap, error::Error};

fn main()  -> Result<(), Box<dyn Error>> {
    let test = "www.yahoo.co.jp";
    let mut headers = HashMap::new();
    headers.insert("Host".to_string(), test.to_string());
    headers.insert("User-Agent".to_string(), "orezia-browser/0.0".to_string());
    let response = http_client::Request::new(
        String::from(test),
        443,
        String::from("/"),
        http_client::Method::GET,
        headers,
        String::from("")).send()?;
    println!("{}", &response.status);
    println!("{:?}", &response.headers);
    println!("{}", &response.body);

    let dom = html_parser::parse( response.body);

    println!("{:?}", dom);

    renderer::render(dom);
    Ok(())
}
