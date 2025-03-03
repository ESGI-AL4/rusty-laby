use serde_json::json;
use std::io;
use std::net::TcpStream;
use crate::bin::network;
use crate::bin::json_utils;

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
        let msg = self.build_register_message();
        network::send_message(&mut self.stream, &msg)?;
        println!("Registration message sent!");

        self.wait_for_token()
    }

    fn build_register_message(&self) -> String {
        json!({
            "RegisterTeam": {
                "name": self.team_name
            }
        })
            .to_string()
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
                }
                Err(e) => println!("Failed to parse JSON: {}", e),
            }
        }
    }

    pub fn subscribe_player(
        &mut self,
        player_name: &str,
        registration_token: &str,
        mut stream: TcpStream,
    ) -> io::Result<String> {
        let msg = self.build_subscribe_message(player_name, registration_token);
        // println!("Server to send: {}", msg);
        network::send_message(&mut stream, &msg)?;
        println!("✅ Sent SubscribePlayer, waiting for server response...");
        self.wait_for_subscription_result(&mut stream)
    }

    fn build_subscribe_message(&self, player_name: &str, registration_token: &str) -> String {
        json!({
            "SubscribePlayer": {
                "name": player_name,
                "registration_token": registration_token
            }
        })
            .to_string()
    }

    fn wait_for_subscription_result(&mut self, stream: &mut TcpStream) -> io::Result<String> {
        loop {
            let msg = network::receive_message(stream)?;
            println!("Server - from subscription: {}", msg);
            let parsed_msg = json_utils::parse_json(&msg)?;
            println!(
                "Server - parsed subscription response: {}",
                parsed_msg["SubscribePlayerResult"]
            );
            return Ok(parsed_msg["SubscribePlayerResult"].to_string());
        }
    }
}

fn main() {}