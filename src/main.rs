mod http_client;

use std::{collections::HashMap, error::Error};

fn main()  -> Result<(), Box<dyn Error>> {
    let mut headers = HashMap::new();
    headers.insert(String::from("test"), String::from("test"));
    let response = http_client::Request::new(
        String::from("www.google.com:80"),
        String::from("/"),
        http_client::Method::GET,
        headers,
        String::from("")).send()?;
    println!("{}", &response.status);

    Ok(())
}
