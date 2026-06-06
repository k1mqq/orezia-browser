use std::collections::HashMap;
use std::error::Error;
use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpStream;
use std::num::ParseIntError;

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
    pub addr: String,
    pub path: String,
    pub method: Method,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request{
    pub fn new(addr: String, path: String, method: Method, headers: HashMap<String, String>, body: String) -> Self{
        Self {
            addr, path, method, headers, body
        }
    }

    fn message(&self) -> Vec<u8> {
        let mut field = String::new();
        for (key ,value) in &self.headers {
            field.push_str(&format!("{}: {}\r\n", key, value).to_string());
        }
        let message = format!("{} {} HTTP/1.1\r\n{}\r\n", self.method.as_str(), self.path, field);
        message.into_bytes()
    }

    pub fn send(&self) -> Result<Response, Box<dyn Error>> {
        let mut stream = TcpStream::connect(&self.addr)?;
        println!("connected!");

        let message= self.message();

        stream.write_all(&message)?;

        let mut buffer= BufReader::new(&stream);

        let mut status_line = String::new();
        let response_headers;

        buffer.read_line(&mut status_line)?;

        let status_code = parse_status_line(status_line)?;

        let mut body = String::new();

        match status_code {
            200 => {
                // more than 100 headers -> error
                // most browsers have 40KB limit on header size
                // does not care transfer encoding
                let mut valid_field = false;
                let mut field = String::new();

                for _ in 1..101 {
                    let mut field_line = String::new();
                    buffer.read_line( &mut field_line)?;

                    if field_line.replace("\r\n", "").is_empty() {
                        valid_field = true;
                        break;
                    }

                    field.push_str(&field_line);
                }

                match valid_field {
                    true => {
                        response_headers = parse_field(field)?;
                    },
                    false => {
                        return Err("too many headers!".into())
                    }
                }
            },
            _ => {
                println!("{}! NOOOOOO", status_code);
                return Err("status code is not 200".into());
            }

        }

        for line in buffer.lines(){
            let line = line?;

            if line.replace("\r\n", "").is_empty() {
                break;
            }
            body.push_str(&line);
        }

        Ok(Response{
            status: status_code,
            headers: response_headers,
            body: body,
        })
    }
}

fn parse_status_line(status_line: String) -> Result<u16, ParseIntError> {
    let v: Vec<&str> = status_line.split(" ").collect();
    v[1].parse()
}

fn parse_field(field: String) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut headers = HashMap::new();

    for line in field.lines() {
        let (key, value) = line.split_once(": ").ok_or("parse failed")?;
        headers.insert(key.to_string(), value.to_string());
        // println!("{}: {}", key, value);
    }

    Ok(headers)
}