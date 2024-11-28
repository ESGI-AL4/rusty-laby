use std::io;
use std::net::TcpStream;
use rusty_laby::TeamRegistration;

const ADDRESS: &str = "localhost:8778";

fn main() -> io::Result<()> {
    let stream = TcpStream::connect(ADDRESS)?;
    println!("Connected to server...");

    let mut registration = TeamRegistration::new("rusty-octopusooooo", stream);

    match registration.register() {
        Ok(token) => println!("Registration successful. Token: {}", token),
        Err(e) => eprintln!("Registration failed: {}", e),
    }

    Ok(())
}

