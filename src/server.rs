use std::net::{TcpListener, TcpStream};
use rusty_laby::bin::network;
use rusty_laby::bin::json_utils;
use rusty_laby::bin::radarview::PrettyRadarView;
use rusty_laby::bin::labyrinth_mock::get_labyrinth_mock;
use serde_json::json;
use std::io;
use std::sync::Arc;

fn receive_and_parse_message(stream: &mut TcpStream) -> io::Result<serde_json::Value> {
    let msg = network::receive_message(stream)?;
    println!("Raw received message: {:?}", msg);
    json_utils::parse_json(&msg)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("JSON parse error: {}", e)))
}

fn answer(stream: &mut TcpStream, response: serde_json::Value) {
    network::send_message(stream, &response.to_string()).unwrap();
    println!("Message sent to client : {:?}", response);
}

fn answer_radar_view(stream: &mut TcpStream, radar_view: &PrettyRadarView) {
    let encoded = format!("{:?}", radar_view);
    answer(stream, json!({"RadarView": encoded}));
}

fn handle_client(mut stream: TcpStream) {
    let mock_views = Arc::new(get_labyrinth_mock());
    let mut current_view = 0;
    let mut game_started = false;

    loop {
        let parsed_msg = match receive_and_parse_message(&mut stream) {
            Ok(msg) => msg,
            Err(_) => break,
        };

        // Handle RegisterTeam
        if parsed_msg.get("RegisterTeam").is_some() {
            answer(&mut stream, json!({
                "RegisterTeamResult": {
                    "Ok": {"expected_players": 3, "registration_token": "SECRET"}
                }
            }));
            continue;
        }

        // Handle SubscribePlayer
        if parsed_msg.get("SubscribePlayer").is_some() {
            answer(&mut stream, json!({ "SubscribePlayerResult": "Ok" }));

            if !game_started {
                game_started = true;
                // Send first mock view
                if let Some(view) = mock_views.get(current_view) {
                    answer_radar_view(&mut stream, view);
                }
            }
            continue;
        }

        // Handle MoveTo actions
        if let Some(action_value) = parsed_msg.get("MoveTo") {
            if let Some(action) = action_value.as_str() {
                // Simple mock progression through views
                current_view = (current_view + 1) % mock_views.len();

                // Send next mock view
                if let Some(view) = mock_views.get(current_view) {
                    answer_radar_view(&mut stream, view);
                }

                answer(&mut stream, json!({"status": "OK"}));
                continue;
            }
        }

        println!("Unknown message: {:?}", parsed_msg);
    }
}

fn main() {
    let listener = TcpListener::bind("localhost:8778").unwrap();
    println!("Server started");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }
}