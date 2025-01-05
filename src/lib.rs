use std::io;
use std::io::{Write, BufRead, BufReader};
use std::net::TcpStream;
use rand::seq::IndexedRandom;
use serde_json::{Value, json};
pub const ADDRESS: &str = "localhost:8778";

use base64::{
    alphabet::Alphabet,
    engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig},
    engine::DecodePaddingMode,
    DecodeError, Engine as _,
};

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

enum Direction {
    FRONT,
    BACK,
    RIGHT,
    LEFT
}

/// Alphabet imposé par le sujet du projet:
///  0..25  -> a..z
///  26..51 -> A..Z
///  52..61 -> 0..9
///  62     -> +
///  63     -> /

static CUSTOM_ALPHABET_STR: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";

fn decode_custom_b64(s: &str) -> Result<Vec<u8>, DecodeError> {
    // 1) Construire l'alphabet custom
    let alphabet = match Alphabet::new(CUSTOM_ALPHABET_STR) {
        Ok(a) => a,
        Err(_parse_err) => {
            // Choisir un variant de DecodeError qui vous convient
            return Err(DecodeError::InvalidByte(0, b'?'));
        }
    };

    // 2) Configurer un mode "indulgent"
    //    - Pas besoin de padding = '='
    //    - Trailing bits autorisés si la longueur n'est pas multiple de 4
    let config = GeneralPurposeConfig::new()
        .with_decode_padding_mode(DecodePaddingMode::Indifferent)
        .with_decode_allow_trailing_bits(true);

    // 3) Créer le moteur qui utilise cet alphabet et cette config
    let engine = GeneralPurpose::new(&alphabet, config);

    // 4) Décoder la chaîne
    engine.decode(s)
}

fn decode_radar_view(
    radar_b64: &str
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let bytes = decode_custom_b64(radar_b64)?;

    // On suppose 11 octets (3+3+5)
    if bytes.len() < 11 {
        return Err(format!("RadarView: {} octets reçus, on s’attend à 11", bytes.len()).into());
    }

    // Extraire
    let horizontals = bytes[0..3].to_vec();
    let verticals   = bytes[3..6].to_vec();
    let cells       = bytes[6..11].to_vec();

    Ok((horizontals, verticals, cells))
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
            ],
        }
    }

    fn decide_next_action(&self, ) -> serde_json::Value {
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

    fn process_radar_view(&mut self, radar_str: &str) {
        match decode_radar_view(radar_str) {
            Ok((horizontals, verticals, cells)) => {
                println!("=== Decoded RadarView ===");
                println!("Horizontals (3 octets): {:?}", horizontals);
                println!("Verticals   (3 octets): {:?}", verticals);
                println!("Cells       (5 octets): {:?}", cells);
                println!("=========================");
            }
            Err(e) => {
                eprintln!("Erreur lors du décodage du RadarView: {}", e);
            }
        }
    }

    /// Boucle principale
    pub fn handle(&mut self) -> io::Result<()> {
        loop {
            // (1) On lit un message depuis le serveur
            let parsed_msg = self.receive_and_parse_message()?;

            // (2) S’il contient un ActionError, on le gère
            if let Some(action_error) = parsed_msg.get("ActionError") {
                println!("ActionError - from server: {:?}", action_error);
                if action_error == "CannotPassThroughWall" {
                    // On pourrait changer de direction, ou autre
                    // Juste un log
                    println!("Impossible de passer: mur");
                    // On continue la boucle, on n’envoie pas forcément d’autre action
                    continue;
                } else {
                    // Pour tout autre ActionError, on quitte
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Exiting due to action error: {:?}", action_error),
                    ));
                }
            }

            // (3) S’il contient un RadarView, on le décode
            if let Some(radar_value) = parsed_msg.get("RadarView") {
                if let Some(radar_str) = radar_value.as_str() {
                    self.process_radar_view(radar_str);
                }
            }

            // (4) Ici, on décide de notre prochaine action
            //     (Ex: on bouge "Front", "Back", etc.)
            let action = self.decide_next_action();
            self.send_action(&action)?;
        }
    }
}

fn encode_custom_b64(bytes: &[u8]) -> String {
    let alphabet = Alphabet::new(CUSTOM_ALPHABET_STR)
        .expect("Invalid custom alphabet?");

    // On configure le moteur
    let mut config = GeneralPurposeConfig::new();
    // Désactiver le padding lors de l'encodage
    config = config.with_encode_padding(false);

    // On peut conserver les règles decode :
    config = config
        .with_decode_padding_mode(DecodePaddingMode::Indifferent)
        .with_decode_allow_trailing_bits(true);

    let engine = GeneralPurpose::new(&alphabet, config);
    engine.encode(bytes)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        // 1) Vérifier l’encodage de quelques octets
        assert_eq!(encode_custom_b64(&[0]), "aa");
        assert_eq!(encode_custom_b64(&[25]), "gq");
        assert_eq!(encode_custom_b64(&[26]), "gG");
        assert_eq!(encode_custom_b64(&[51]), "mW");
        assert_eq!(encode_custom_b64(&[52]), "na");
        assert_eq!(encode_custom_b64(&[61]), "pq");
        assert_eq!(encode_custom_b64(&[62]), "pG");
        assert_eq!(encode_custom_b64(&[63]), "pW");

        // "Hello, World!" => "sgvSBg8SifDVCMXKiq"
        assert_eq!(encode_custom_b64(b"Hello, World!"), "sgvSBg8SifDVCMXKiq");

        // 2) Un tableau plus gros (0..=255)
        let big_array: Vec<u8> = (0..=255).collect();
        let encoded = encode_custom_b64(&big_array);
        assert_eq!(
            encoded,
            "aaecaWqfbGCicqOlda0odXareHmufryxgbKAgXWDhH8GisiJjcuMjYGPkISSls4VmdeYmZq1nJC4otO7pd0+p0bbqKneruzhseLks0XntK9quvjtvfvwv1HzwLTCxv5FygfIy2rLzMDOAwPRBg1UB3bXCNn0Dxz3EhL6E3X9FN+aGykdHiwgH4IjIOUmJy6pKjgsK5svLPEyMzQBNj2EN6cHOQoKPAANQkMQQ6YTRQ+WSBkZTlw2T7I5URU8VB6/WmhcW8tfXSFiYCRlZm3oZ9dr0Tpu1DBx2nNA29ZD3T/G4ElJ5oxM5+JP6UVS7E7V8phY8/t19VF4+FR7/p3+/W"
        );

        // 3) Tests encode + decode “Hello Radar!”
        let test_string = "Hello Radar!";
        let encoded = encode_custom_b64(test_string.as_bytes());
        let decoded = decode_custom_b64(&encoded).unwrap();
        assert_eq!(decoded, test_string.as_bytes());

        // 4) Test sur string vide
        let test_string = "";
        let encoded = encode_custom_b64(test_string.as_bytes());
        let decoded = decode_custom_b64(&encoded).unwrap();
        assert_eq!(decoded, test_string.as_bytes());

        // 5) Test sur string plus longue
        let test_string = "This is a longer test string to really see if everything is working fine.";
        let encoded = encode_custom_b64(test_string.as_bytes());
        let decoded = decode_custom_b64(&encoded).unwrap();
        assert_eq!(decoded, test_string.as_bytes());

        println!("All encode/decode tests passed successfully!");
    }
}
