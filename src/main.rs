use log::{error, debug, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

extern crate webapp;
use webapp::ThreadPool;

fn main() {
    let logfile = FileAppender::builder()
        .build("logs/activity.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Trace))
        .unwrap();
    
    let _handle = log4rs::init_config(config).unwrap();

    let listener = match TcpListener::bind("127.0.0.1:7878"){
        Ok(listener) => {
            debug!("Successfully bound to port: 127.0.0.1:7878.");
            listener
        },
        Err(e) => { 
            error!("{}, SHUTTING DOWN.", e); 
            std::process::exit(1)
        }
    };

    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(4) {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    debug!("Now handling TcpStream client: {}", stream.peer_addr().unwrap());
                    handle_connection(stream);
                });
            }
            Err(e) => { 
                error!("{}, SHUTTING DOWN.", e); 
                std::process::exit(1)
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(_) => (),
        Err(e) => {
            error!("{}, SHUTTING DOWN.", e); 
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

    let contents = match fs::read_to_string(filename) {
        Ok(str) => str,
        Err(e) => {
            error!("Failed to read query contents to string: {}, SHUTTING DOWN.", e); 
            std::process::exit(1)
        },
    };

    let response = format!{
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    };
    

    match stream.write_all(response.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            error!("Couldn't write contents of buffer: {}, SHUTTING DOWN.", e); 
            std::process::exit(1)
        },
    };

    match stream.flush() {
        Ok(_) => (),
        Err(e) => {
            error!("Couldn't flush contents of buffer: {}, SHUTTING DOWN.", e); 
            std::process::exit(1)
        },
    };
}