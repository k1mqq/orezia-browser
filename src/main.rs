mod http_client;

use std::collections::HashMap;

fn main() {
    let mut headers = HashMap::new();
    headers.insert(String::from("test"), String::from("test"));
    let response = http_client::Request::new(
        String::from("127.0.0.1:8000"),
        String::from("/.git/"),
        http_client::Method::GET,
        headers,
        String::from("")).send();

}
