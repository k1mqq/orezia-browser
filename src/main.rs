mod app;
mod css_parser;
mod html_parser;
mod http_client;
mod layout;
mod renderer;
mod styler;
mod url;

use std::{env, error::Error};

use crate::css_parser::StyleSheet;
use crate::html_parser::{Dom, NodeType};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let url = &args[1];

    let response = http_client::get(url)?;
    println!("{}", &response.status);
    println!("{:?}", &response.headers);
    println!("{}", &response.body);

    let dom = html_parser::parse(response.body);

    dom.print(0, 0);

    let style_sheets = fetch_css(&dom, url);

    let styled_tree = styler::StyledTree::build(&dom, &style_sheets);

    styled_tree.print(0, 0);

    app::init(styled_tree);
    Ok(())
}

fn fetch_css(dom: &Dom, url: &str) -> Vec<StyleSheet> {
    let mut style_sheets = Vec::new();

    let link_ids = dom.get_element_by_tag_name("link");
    println!("{:?}", link_ids);

    // i don't like this code
    link_ids.iter().for_each(|&id| {
        println!("{:?}", dom.nodes[id].node_type);
        if let NodeType::Element { tag: _, attributes } = &dom.nodes[id].node_type {
            if attributes
                .iter()
                .any(|(key, value)| key == "rel" && value == "stylesheet")
            {
                for (key, value) in attributes {
                    if key == "href" {
                        let fetch_url = format!("{}{}", url, value);
                        let res = http_client::get(&fetch_url).expect("cannnot fetch external css");
                        style_sheets.push(css_parser::parse(&res.body));
                        println!("{:?}", css_parser::parse(&res.body));
                    }
                }
            }
        }
    });

    style_sheets
}
