use std::io::{self};
use std::net::TcpStream;
use rusty_laby::{GameStreamHandler, TeamRegistration, ADDRESS};

#[allow(unused_variables)]
fn main() -> io::Result<()> {
    let stream = TcpStream::connect(ADDRESS)?;
    println!("Connected to server...");

    let mut registration = TeamRegistration::new("rusty-ocho", stream);
    let token = registration.register()?;
    println!("Registration successful. Token: {}", token);

    let game_stream = TcpStream::connect(ADDRESS)?;
    registration.subscribe_player("rusty_player", &token, game_stream.try_clone()?)?;
    println!("Player subscribed successfully!");

    let mut game_stream = GameStreamHandler::new(game_stream);
    let _ = GameStreamHandler::handle(&mut game_stream);

    Ok(())
}

