use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

pub fn start_server(address: &str) {
    let listener = TcpListener::bind(address).expect("Failed to bind to address");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 512];
                stream.read(&mut buffer).unwrap();
                println!("Received: {}", String::from_utf8_lossy(&buffer));
                stream.write(b"Hello, from Crabby!").unwrap();
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }
}
