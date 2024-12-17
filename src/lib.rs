use std::io;
use std::io::{Write, BufRead, BufReader};
use std::net::TcpStream;
use rand::seq::IndexedRandom;
use serde_json::{Value, json};
pub const ADDRESS: &str = "localhost:8778";

pub mod network {
    use std::io::Read;
    use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
    use super::*;

    pub fn send_message(stream: &mut TcpStream, message: &str) -> io::Result<()> {
        let message_bytes = message.as_bytes();
        let size = message_bytes.len() as u32;

        stream.write_u32::<LittleEndian>(size)?;
        stream.write_all(message_bytes)?;
        Ok(())
    }

    pub fn receive_message(stream: &mut TcpStream) -> io::Result<String> {
        let size = stream.read_u32::<LittleEndian>()?;

        let mut buffer = vec![0; size as usize];
        stream.read_exact(&mut buffer)?;

        String::from_utf8(buffer).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid data: {}", e))
        })
    }

    pub fn connect_to_server(address: &str) -> io::Result<TcpStream> {
        TcpStream::connect(address)
    }
}

pub mod json_utils {
    use super::*;

    pub fn parse_json(msg: &str) -> Result<Value, serde_json::Error> {
        serde_json::from_str(msg)
    }

    pub fn extract_registration_token(json: &Value) -> Option<&str> {
        json.get("RegisterTeamResult")?
            .get("Ok")?
            .get("registration_token")?
            .as_str()
    }
}

pub struct TeamRegistration {
    team_name: String,
    stream: TcpStream,
}

impl TeamRegistration {
    pub fn new(team_name: &str, stream: TcpStream) -> Self {
        Self {
            team_name: team_name.to_string(),
            stream,
        }
    }

    pub fn register(&mut self) -> io::Result<String> {
        let message = self.build_register_message();
        network::send_message(&mut self.stream, &message)?;
        println!("Registration message sent!");

        self.wait_for_token()
    }

    fn build_register_message(&self) -> String {
        json!({
            "RegisterTeam": {
                "name": self.team_name
            }
        }).to_string()
    }

    fn wait_for_token(&mut self) -> io::Result<String> {
        loop {
            let msg = network::receive_message(&mut self.stream)?;
            println!("Server: {}", msg);

            match json_utils::parse_json(&msg) {
                Ok(json) => {
                    if let Some(token) = json_utils::extract_registration_token(&json) {
                        return Ok(token.to_string());
                    }
                },
                Err(e) => println!("Failed to parse JSON: {}", e),
            }
        }
    }

    pub fn subscribe_player(&mut self, player_name: &str, registration_token: &str, mut stream: TcpStream) -> std::io::Result<String> {
        let message = self.build_subscribe_message(player_name, registration_token);
        println!("Server to send: {}", message);
        network::send_message(&mut stream, &message)?;
        println!("Subscribe message sent to server!");
        self.wait_for_subscription_result(&mut stream)
    }

    fn build_subscribe_message(&self, player_name: &str, registration_token: &str) -> String {
        json!({
            "SubscribePlayer": {
                "name": player_name,
                "registration_token": registration_token
            }
        }).to_string()
    }

    fn wait_for_subscription_result(&mut self, stream: &mut TcpStream) -> io::Result<String> {
        loop {
            let msg = network::receive_message(stream)?;
            println!("Server - from subscription: {}", msg);

            let parsed_msg = json_utils::parse_json(&msg)?;
            println!("Server - parsed subscription response: {}", parsed_msg["SubscribePlayerResult"]);

            return Ok(parsed_msg["SubscribePlayerResult"].to_string());
        }
    }

}

pub struct GameStreamHandler {
    stream: TcpStream,
    directions: Vec<String>
}

impl GameStreamHandler {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            directions: vec![
                "Front".to_string(),
                "Right".to_string(),
                "Back".to_string(),
                "Left".to_string(),
            ],}

    }

    fn decide_next_action(&self) -> serde_json::Value {
        // Parse the RadarView to understand the surroundings
        // For now, we assume a simple logic: always move "Right"
        // In the future, this will involve analyzing the radar_view content.
        // serde_json::Value::String("MoveTo".to_string())
        let mut rng = rand::rng();
        let default_direction = "Front".to_string();
        let random_direction = self.directions.choose(&mut rng).unwrap_or(&default_direction);
        println!("Decide next action: {}", random_direction);

        json!({"MoveTo" : random_direction})
    }
    fn receive_and_parse_message(&mut self) -> io::Result<serde_json::Value> {
        let msg = network::receive_message(&mut self.stream)?;
        println!("Server - received message: {}", msg);

        let parsed_msg = json_utils::parse_json(&msg)?;

        Ok(parsed_msg)
    }
    fn send_action(&mut self, action: &serde_json::Value) -> io::Result<()> {
        let action_request = json!({ "Action": action }).to_string();
        println!("Client - Action to server: {}", action_request);
        network::send_message(&mut self.stream, &action_request)?;

        Ok(())
    }
    fn handle_action_response(&mut self) -> io::Result<Option<String>> {
        let response = network::receive_message(&mut self.stream)?;
        println!("Server - action response: {}", response);

        let parsed_response = json_utils::parse_json(&response)?;
        if let Some(action_error) = parsed_response.get("ActionError") {
            println!("ActionError - Action error from server: {:?}", action_error);

            if action_error == "CannotPassThroughWall" {
                return Ok(Some(action_error.to_string()));
            }

            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Exiting due to action error: {:?}", action_error),
            ));
        }

        Ok(None)
    }


    pub fn handle(&mut self) -> io::Result<()> {
        loop {
            let action = self.decide_next_action();
            self.send_action(&action)?;

            // let parsed_msg = self.receive_and_parse_message()?;
            //
            // if let Some(radar_view) = parsed_msg.get("RadarView") {
            //     println!("RadarView received: {:?}", radar_view);
            //     let action = self.decide_next_action(radar_view);
            //     self.send_action(&action)?;
            //
            //     self.handle_action_response()?;
            // }
        }
    }
}
