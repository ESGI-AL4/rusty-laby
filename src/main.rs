use std::net::{TcpStream};

fn main() {
    if let Ok(stream) = TcpStream::connect("localhost:8778") {
        println!("Connected to the server!");
    } else {
        println!("Couldn't connect to server...");
    }}
