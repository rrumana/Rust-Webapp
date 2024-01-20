use std::fs;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream)?;
            }
            Err(e) => {
                eprintln!("{}", e); 
                std::process::exit(1)
            },
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];

    let _result = match stream.read(&mut buffer) {
        Ok(buf) => buf,
        Err(e) => {
            eprintln!("{}", e); 
            std::process::exit(1)
        },
    };

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = 
        if buffer.starts_with(get) {
            ("HTTP/1.1 200 OK", "html/index.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "html/404.html")
        };

    let contents = fs::read_to_string(filename)?;

    let response = format!{
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    };
    
    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}