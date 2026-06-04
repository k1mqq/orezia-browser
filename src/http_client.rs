use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpStream;

pub enum Method{
    GET, HEAD, POST, PUT, DELETE, CONNECT, OPTIONS, TRACE, PATCH
}

impl Method{
    fn as_str(&self) -> &'static str {
        match self{
            Self::GET => "GET",
            Self::HEAD => "HEAD",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::CONNECT => "CONNECT",
            Self::OPTIONS => "OPTIONS",
            Self::TRACE => "TRACE",
            Self::PATCH => "PATCH",
        }
    }
}
pub struct Request {
    addr: String,
    path: String,
    method: Method,
    headers: HashMap<String, String>,
    body: String,
}

pub struct Response {
    status: u8,
    headers: HashMap<String, String>,
    body: String,
}

impl Request{
    pub fn new(addr: String, path: String, method: Method, headers: HashMap<String, String>, body: String) -> Self{
        Self {
            addr, path, method, headers, body
        }
    }

    fn message(&self) -> Vec<u8> {
        let message = format!("{} {} HTTP/1.1\r\n\r\n", self.method.as_str(), self.path);
        message.into_bytes()
    }

    pub fn send(&self) -> Result<Response, Box<dyn Error>> {
        let mut stream = TcpStream::connect(&self.addr)?;
        println!("connected!");

        let message= self.message();

        stream.write_all(&message)?;

        let buffer= BufReader::new(&stream);
        for line in buffer.lines(){
            let line = line?;
            println!("{}", line);
        }

        let mut response_headers = HashMap::new();
        response_headers.insert(String::from("Test"), String::from("test"));

        Ok(Response{
            status: 200,
            headers: response_headers,
            body: String::from("this is body")
        })
    }
}