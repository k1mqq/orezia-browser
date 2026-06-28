mod http_client;
mod html_parser;
mod renderer;
mod layout;
mod url;

use std::{collections::HashMap, env, error::Error};

fn main()  -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let url = url::URL::parse(args[1].to_string()).unwrap();
    println!("{:?}", url);
    let port = match url.port {
        Some(n) => n,
        None => {
            match url.scheme.as_str() {
                "https" => 443,
                _ => 80
            }
        }
    };

    let mut headers = HashMap::new();
    headers.insert("Host".to_string(), url.host.to_string());
    headers.insert("User-Agent".to_string(), "orezia-browser/0.0".to_string());
    headers.insert("Accept-Encoding".to_string(), "identity".to_string());
    let response = http_client::Request::new(
        String::from(&url.host),
        port,
        format!("/{}", url.path),
        http_client::Method::GET,
        headers,
        String::from("")).send()?;
    println!("{}", &response.status);
    println!("{:?}", &response.headers);
    println!("{}", &response.body);

    let dom = html_parser::parse( response.body);

    dom.print(0, 0);

    renderer::render(dom);
    Ok(())
}
