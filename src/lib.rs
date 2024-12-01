use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json::{Value, json};
pub const ADDRESS: &str = "localhost:8778";

pub mod network {
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
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e))
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

    pub fn subscribe_player(&mut self, player_name: &str, registration_token: &str, mut stream: TcpStream) -> io::Result<()> {
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

    fn wait_for_subscription_result(&mut self, stream: &mut TcpStream) -> io::Result<()> {
        loop {
            let msg = network::receive_message(stream)?;
            println!("Server: {}", msg);

            match json_utils::parse_json(&msg) {
                Ok(json) => {
                    if let Some(result) = json.get("SubscribePlayerResult") {
                        match result.get("Ok") {
                            Some(_) => {
                                println!("Player subscription successful!");
                                return Ok(());
                            }
                            None => {
                                if let Some(err) = result.get("Err") {
                                    eprintln!("Player subscription failed: {}", err);
                                    return Err(io::Error::new(io::ErrorKind::Other, "Subscription failed"));
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("Failed to parse JSON: {}", e),
            }
        }
    }
}
