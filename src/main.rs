use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use std::thread;

use hulloserver::ThreadPool;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming(){
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_cnx(stream);
        });
        //println!("i'm in");
    }

    println!("main shutting down");
}

fn handle_cnx(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();

    let get1 = b"GET / HTTP/1.1\r\n";
    let get0 = b"GET / HTTP/1.0\r\n";
    
    let sleep1 = b"GET /sleep HTTP/1.1\r\n";
    let sleep0 = b"GET /sleep HTTP/1.0\r\n";
    
    let (status_line, fname) = if buf.starts_with(get1) || buf.starts_with(get0) {
        ("HTTP/1.0 200 OK", "hullo.html")
    } else if buf.starts_with(sleep1) || buf.starts_with(sleep0) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.0 200 OK", "hullo.html")
    } else {
        ("HTTP/1.0 404 NOT FOUND", "404.html")
    };

    let ctnt = fs::read_to_string(fname).unwrap();

    let resp = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        ctnt.len(),
        ctnt
        );

    //println!("Request: {}", String::from_utf8_lossy(&buf[..]));
    stream.write(resp.as_bytes()).unwrap();
    stream.flush().unwrap();
}
