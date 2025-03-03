use std::io::{self, BufRead};
use std::net::TcpStream;
use std::thread;
use rusty_laby::{GameStreamHandler, ADDRESS};
use std::sync::Arc;
use std::sync::RwLock;
use std::env;

use rusty_laby::bin::team_registration::TeamRegistration;
use rusty_laby::bin::challengehandler::ChallengeHandler;

#[allow(unused_variables)]
fn main() -> io::Result<()> {
    let stream = TcpStream::connect(ADDRESS)?;
    println!("Connected to server...");
    let mut getTeamSize = false;
    let mut players_number = 3;
    let mut ui_enabled = false;
    let mut game_winned = true;
    //Handle params
    for arg in env::args() {
        if getTeamSize {
            players_number = arg.parse().unwrap();
            getTeamSize = false;
        }
        if arg == "--team-size" {
            getTeamSize = true;
        }

        if arg == "--ui" {
            ui_enabled = true;
        }
    }
    let mut registration = TeamRegistration::new("rusty-ocho", stream);
    let token = registration.register()?;
    println!("Registration successful. Token: {}", token);

    

    let challenge_handler = Arc::new(RwLock::new(ChallengeHandler::new(players_number)));

    if players_number >= 2 {
    let mut handlers = vec![];
        for i in 0..players_number {
            let name = format!("rusty_player{}", i);
            let game_stream = TcpStream::connect(ADDRESS)?;
            registration.subscribe_player(name.as_str(), &token, game_stream.try_clone()?)?;
            let player_challenge_handler = challenge_handler.clone();
            println!("Player {} subscribed successfully!", name);
            let handler = thread::spawn(move || {
                let mut game_stream = GameStreamHandler::new(game_stream, player_challenge_handler, i, false);
                let _ = GameStreamHandler::handle(&mut game_stream);
            });
            handlers.push(handler);
        }

        for handler in handlers {
            match handler.join() {
                Err(e) => {
                    println!("Error: {:?}", e);
                    game_winned = false;
                },
                _ => {}
            }
        }
    } else {
        let game_stream = TcpStream::connect(ADDRESS)?;
        registration.subscribe_player("rusty_player", &token, game_stream.try_clone()?)?;
        println!("Player subscribed successfully!");

        let mut game_stream = GameStreamHandler::new(game_stream, challenge_handler, 0, ui_enabled);
        if ui_enabled {
            game_stream.init_piston();
        }
        // *** On init Piston ici ***
        let result = GameStreamHandler::handle(&mut game_stream);
        match result {
            Err(e) =>{
                println!("Error: {}", e);
                game_winned = false;
            },
            _ => {}
        }
    }
    if game_winned {
        println!("Game wonned!");
    } else {
        println!("Game lost!");
    }
    Ok(())
}

