use std::collections::HashMap;
use std::error::Error;
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
        let status_code = build_status_code(&mut buffer)?;

        let response_headers = build_headers(&mut buffer)?;

        let mut body = String::new();

        match status_code {
            100 => {},
            101 => {},
            102 => {},
            103 => {},

            200 => {},
            201 => {},
            202 => {},
            203 => {},
            204 => {},
            205 => {},
            206 => {},

            300 => {},
            301 => {},
            302 => {},
            303 => {},
            304 => {},
            305 => {},
            306 => {},
            307 => {},
            308 => {},

            400 => {},
            401 => {},
            402 => {},
            403 => {},
            404 => {},
            405 => {},
            406 => {},
            407 => {},
            408 => {},
            409 => {},
            410 => {},
            411 => {},
            412 => {},
            413 => {},
            414 => {},
            415 => {},
            416 => {},
            417 => {},
            418 => {},
            419 => {},
            420 => {},
            421 => {},
            422 => {},
            423 => {},
            424 => {},
            425 => {},
            426 => {},
            428 => {},
            429 => {},
            431 => {},
            451 => {},

            500 => {},
            501 => {},
            502 => {},
            503 => {},
            504 => {},
            505 => {},
            506 => {},
            507 => {},
            508 => {},
            509 => {},
            510 => {},
            511 => {},

            _ => {
                println!("{}! NOOOOOO", status_code);
                return Err("Received unknown status code".into());
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

fn build_status_code(buffer: &mut BufReader<&TcpStream>) -> Result<u16, Box<dyn Error>> {
    let mut status_line = String::new();
    buffer.read_line(&mut status_line)?;

    parse_status_line(status_line)
}

fn parse_status_line(status_line: String) -> Result<u16, Box<dyn Error>> {
    let v: Vec<&str> = status_line.split(" ").collect();
    let i: u16 = v[1].parse()?;
    Ok(i)
}

fn build_headers(buffer: &mut BufReader<&TcpStream>) -> Result<HashMap<String, String>, Box<dyn Error>> {
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
            parse_field(field)
        },
        false => {
            Err("too many headers!".into())
        }
    }
}

fn parse_field(field: String) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut headers = HashMap::new();

    for line in field.lines() {
        let (key, value) = line.split_once(": ").ok_or("parse failed")?;
        headers.insert(key.to_string(), value.to_string());
    }

    Ok(headers)
}