use std::collections::HashMap;
use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpStream;

#[derive(Debug)]
pub enum HttpError {
    Connection(std::io::Error),
    Parse{
        raw: String,
        message: String,
    },
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::Connection(e) => write!(f, "connection error: {e}"),
            HttpError::Parse { raw, message} => write!(f, "{message}\nraw: {raw}"),
        }
    }
}

impl From<std::io::Error> for HttpError {
    fn from(err: std::io::Error) -> HttpError {
        HttpError::Connection(err)
    }
}
impl std::error::Error for HttpError {}

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
        let message = format!("{} {} HTTP/1.1\r\n{}{}\r\n", self.method.as_str(), self.path, field, self.body);
        message.into_bytes()
    }

    pub fn send(&self) -> Result<Response, HttpError> {
        let mut stream = TcpStream::connect(&self.addr)?;
        println!("connected!");

        let message= self.message();

        stream.write_all(&message)?;

        let mut buffer= BufReader::new(&stream);
        let status_code = build_status_code(&mut buffer)?;

        let response_headers = build_headers(&mut buffer)?;

        let body = if let Some(_) = response_headers.get("Transfer-Encoding") {
            build_body_chunked(&mut buffer)?
        } else if let Some(content_length) = response_headers.get("Content-Length") {
            let length= usize::from_str_radix(&content_length, 10).map_err(
                |_| HttpError::Parse {
                    message: "invalid content-length".to_string(), 
                    raw: content_length.to_string(),
                }
            )?;
            build_body(&mut buffer, length)?
        } else {
            return Err(HttpError::Parse { raw: "".to_string(), message: "response did not have transfer-encoding or content-length".to_string() });
        };

        Ok(Response{
            status: status_code,
            headers: response_headers,
            body: body,
        })
    }
}

fn build_body(buffer: &mut BufReader<&TcpStream>, length: usize) -> Result<String, HttpError> {
    let mut body = String::new();
    let mut buf = vec![0u8; length];
    buffer.read_exact(&mut buf)?;

    body.push_str(&String::from_utf8(buf).map_err(|_| HttpError::Parse{
        raw: "".to_string(),
        message: "invalid body".to_string(),
    })?);
    
    Ok(body)
}

fn build_body_chunked(buffer: &mut BufReader<&TcpStream>) -> Result<String, HttpError> {
    let mut body = String::new();

    // more than 100 chunks -> error
    for _ in 1..101 {
        let mut size_line = String::new();
        buffer.read_line(&mut size_line)?;

        size_line = size_line.replace("\r\n", "");

        let size= usize::from_str_radix(&size_line, 16).map_err(
            |_| HttpError::Parse {
                message: "Chunk size may be too big".to_string(), 
                raw: size_line,
            }
        )?;

        if size == 0 {
            let mut trailing = String::new();
            buffer.read_line(&mut trailing)?;
            return Ok(body);
        }

        let mut buf = vec![0u8; size];

        buffer.read_exact(&mut buf)?;

        let mut trailing = String::new();
        buffer.read_line(&mut trailing)?;

        body.push_str(&String::from_utf8(buf).map_err(|_| HttpError::Parse{
            raw: "".to_string(),
            message: "invalid body".to_string(),
        })?);
    }

    Err(HttpError::Parse { raw: "".to_string(), message: "invalid body".to_string() })
}

fn build_status_code(buffer: &mut BufReader<&TcpStream>) -> Result<u16, HttpError> {
    let mut status_line = String::new();
    buffer.read_line(&mut status_line)?;

    parse_status_line(status_line)
}

fn parse_status_line(status_line: String) -> Result<u16, HttpError> {
    let v: Vec<&str> = status_line.split(" ").collect();
    let i: u16 = v[1].parse().map_err(|_| HttpError::Parse {
        raw: status_line,
        message: "invalid status line".to_string()
    })?;
    Ok(i)
}

fn build_headers(buffer: &mut BufReader<&TcpStream>) -> Result<HashMap<String, String>, HttpError> {
    // more than 100 headers -> error
    // most browsers have 40KB limit on header size
    // does not care transfer encoding for now
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
            Err(HttpError::Parse{
                message: "Too many headers.".to_string(),
                raw: field,
            })
        }
    }
}

fn parse_field(field: String) -> Result<HashMap<String, String>, HttpError> {
    let mut headers = HashMap::new();

    for line in field.lines() {
        let (key, value) = line.split_once(": ").ok_or(HttpError::Parse {
            raw: line.to_string(),
            message: "Parse failed :(".to_string(),
        })?;
        headers.insert(key.to_string(), value.to_string());
    }

    Ok(headers)
}