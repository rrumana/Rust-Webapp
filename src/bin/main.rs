use log::error;
use log::info;
use log::warn;
use log::{debug, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use std::fs;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

extern crate webapp;
use webapp::ThreadPool;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:7878"){
        Ok(listener) => { listener },
        Err(e) => eprintln!("{}", e)
    };

    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(err) if is_fatal(&err) => return Err(err),
            Err(err) => log::error!("failed to accept: {}", err)
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    let _ = match stream.read(&mut buffer) {
        Ok(buf) => buf,
        Err(e) => {
            eprintln!("{}", e); 
            std::process::exit(1)
        },
    };

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = 
        if buffer.starts_with(get) {
            ("HTTP/1.1 200 OK", "html/index.html")
        } else if buffer.starts_with(sleep) {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "html/index.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "html/404.html")
        };

    let contents = fs::read_to_string(filename).expect("Couldn't read page contents.");

    let response = format!{
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    };
    
    stream.write_all(response.as_bytes()).expect("Couldn't write contents of buffer.");
    stream.flush().expect("Couldn't flush contents of buffer.");
}