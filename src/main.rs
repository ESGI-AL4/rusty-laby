use std::io;
use std::net::TcpStream;
use rusty_laby::{TeamRegistration, ADDRESS};

fn main() -> io::Result<()> {
    let stream = TcpStream::connect(ADDRESS)?;
    println!("Connected to server...");

    let mut registration = TeamRegistration::new("rusty-ocho", stream);

    match registration.register() {
        Ok(token) => {
            println!("Registration successful. Token: {}", token);
            let stream = TcpStream::connect(ADDRESS)?;
            match registration.subscribe_player("rusty_player", &token, stream){
                Ok(_) => println!("Subscribed to player"),
                Err(e) => println!("Error subscribing to player: {:?}", e)
            }
        }
        Err(e) => eprintln!("Registration failed: {}", e),
    }

    Ok(())
}

