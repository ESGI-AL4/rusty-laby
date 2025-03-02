use std::io::Read;
use std::net::{TcpListener, TcpStream};
use rusty_laby::bin::network;
use rusty_laby::bin::json_utils;
use rusty_laby::bin::radarview::{decode_radar_view, encode_radar_view, interpret_radar_view, serialize_radar_view, DecodedCell, PrettyRadarView};
use std::io;
use rusty_laby::bin::labyrinth_mock::get_labyrinth_mock;

use rusty_laby::bin::network::receive_message;

fn receive_and_parse_message(stream: &mut TcpStream) -> io::Result<serde_json::Value> {
    let msg = network::receive_message(stream)?;
    println!("Raw received message: {:?}", msg);  // Add this debug log
    let parsed_msg = json_utils::parse_json(&msg)?;
    Ok(parsed_msg)
}


fn answer(stream: &mut TcpStream, response: serde_json::Value) {
    network::send_message(stream, &response.to_string()).unwrap();
}

fn full_encode_radar_view(radarView: &PrettyRadarView) -> String {
    let (h,v,c) = serialize_radar_view(radarView);
    let string = encode_radar_view(&h, &v, &c);
    return string;
}

fn answerRadarView(stream: &mut TcpStream, radarView: &PrettyRadarView) {
    let response = serde_json::json!({"RadarView": full_encode_radar_view(radarView)});
    answer(stream, response);
}


fn handle_client(mut stream: TcpStream) {
    let labyrinth = get_labyrinth_mock();
    let mut actions = 0;
    println!("New connection: {}", stream.peer_addr().unwrap());

    loop {  // 👈 Keep processing messages from the same client
        println!("Waiting for a new message...");

        let parsed_msg = match receive_and_parse_message(&mut stream) {
            Ok(msg) => {
                println!("🔍 Received message: {:?}", msg);
                msg
            }
            Err(err) => {
                println!("❌ Error receiving message: {}", err);
                break; // Exit on error
            }
        };

        if let Some(_registration) = parsed_msg.get("RegisterTeam") {
            let response = serde_json::json!({
                "RegisterTeamResult": {
                    "Ok": { "expected_players": 3, "registration_token": "SECRET" }
                }
            });
            println!("Sending registration response...");
            answer(&mut stream, response);
            continue;
        }

        if let Some(_subscribe) = parsed_msg.get("SubscribePlayer") {
            let response = serde_json::json!({ "SubscribePlayerResult": "Ok" });
            println!("✅ Received SubscribePlayer, sending response...");
            answer(&mut stream, response);
            answerRadarView(&mut stream, &labyrinth[actions]);
            actions += 1;
            continue;
        }
    }
}


fn main() {
    // configure the log level; but default only error
    println!("Server started");
    match init() {
        Ok(_) => {
            println!("server done");
        }
        Err(_) => {
            println!("server error");
        }
    }
}

fn init() -> std::io::Result<()> {
    let listener = TcpListener::bind("localhost:8778")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        println!("New connection");
        let stream = stream.unwrap();
        handle_client(stream);
    }
    println!("server done");
    Ok(())
}





