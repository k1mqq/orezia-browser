mod app;
mod css_parser;
mod html_parser;
mod http_client;
mod layout;
mod renderer;
mod styler;
mod url;

use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let response = http_client::get(&args[1])?;
    println!("{}", &response.status);
    println!("{:?}", &response.headers);
    println!("{}", &response.body);

    let dom = html_parser::parse(response.body);

    dom.print(0, 0);

    app::init(dom);
    Ok(())
}
