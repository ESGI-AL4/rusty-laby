use std::io;
use std::io::{Write, BufRead, BufReader};
use std::net::TcpStream;
use rand::seq::IndexedRandom;
use serde_json::{Value, json};

pub const ADDRESS: &str = "localhost:8778";

/// Bits 3..2 (nature): 00(None), 01(Index=H), 10(Goal=G), 11(Invalid)
#[derive(Debug, Clone, Copy)]
enum CellNature {
    None,
    Index,
    Goal,
    Invalid,
}

/// Bits 1..0 (entité): 00(None), 01(Ally=P), 10(Enemy=O), 11(Monster=M)
#[derive(Debug, Clone, Copy)]
enum CellEntity {
    None,
    Ally,
    Enemy,
    Monster,
}

/// Combine nature + entité sur 4 bits
#[derive(Debug, Clone, Copy)]
struct DecodedCell {
    nature: CellNature,
    entity: CellEntity,
}

// -----------------------------------------------------------------------------
// Imports base64 custom
// -----------------------------------------------------------------------------
use base64::{
    alphabet::Alphabet,
    engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig},
    engine::DecodePaddingMode,
    DecodeError, Engine as _,
};

// -----------------------------------------------------------------------------
// Module réseau
// -----------------------------------------------------------------------------
pub mod network {
    use super::*;
    use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
    use std::io::Read;

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

// -----------------------------------------------------------------------------
// Module JSON
// -----------------------------------------------------------------------------
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

// -----------------------------------------------------------------------------
// TeamRegistration
// -----------------------------------------------------------------------------
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

    pub fn subscribe_player(
        &mut self,
        player_name: &str,
        registration_token: &str,
        mut stream: TcpStream,
    ) -> std::io::Result<String> {
        let msg = self.build_subscribe_message(player_name, registration_token);
        println!("Server to send: {}", msg);
        network::send_message(&mut stream, &msg)?;
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
            println!("Server - parsed subscription response: {}",
                     parsed_msg["SubscribePlayerResult"]);
            return Ok(parsed_msg["SubscribePlayerResult"].to_string());
        }
    }
}

// -----------------------------------------------------------------------------
// Directions
// -----------------------------------------------------------------------------
enum Direction {
    FRONT,
    BACK,
    RIGHT,
    LEFT
}

// -----------------------------------------------------------------------------
// Alphabet custom
// -----------------------------------------------------------------------------
static CUSTOM_ALPHABET_STR: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";

/// Décode la chaîne b64 custom
fn decode_custom_b64(s: &str) -> Result<Vec<u8>, DecodeError> {
    let alphabet = Alphabet::new(CUSTOM_ALPHABET_STR)
        .map_err(|_| DecodeError::InvalidByte(0, b'?'))?;
    let config = GeneralPurposeConfig::new()
        .with_decode_padding_mode(DecodePaddingMode::Indifferent)
        .with_decode_allow_trailing_bits(true);

    let engine = GeneralPurpose::new(&alphabet, config);
    engine.decode(s)
}

/// Inversion horizontals & verticals
fn decode_radar_view(
    radar_b64: &str
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let bytes = decode_custom_b64(radar_b64)?;

    if bytes.len() < 11 {
        return Err(format!("RadarView: {} octets reçus, on s’attend à 11", bytes.len()).into());
    }

    // 3 + 3 + 5
    let mut horizontals = bytes[0..3].to_vec();
    let mut verticals   = bytes[3..6].to_vec();
    let cells           = &bytes[6..11];

    // Inversion => little-endian
    horizontals.reverse();
    verticals.reverse();

    Ok((horizontals, verticals, cells.to_vec()))
}

// -----------------------------------------------------------------------------
// Walls
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
enum Wall {
    Undefined,
    Open,
    Wall,
}

// -----------------------------------------------------------------------------
// decode_walls => 3 octets => 12 passages
fn decode_walls(bytes: &[u8]) -> Vec<Wall> {
    let mut walls = Vec::with_capacity(12);

    let mut bits_24 = 0u32;
    for &b in bytes {
        bits_24 = (bits_24 << 8) | (b as u32);
    }
    for i in 0..12 {
        let shift = 24 - 2 * (i + 1);
        let val = (bits_24 >> shift) & 0b11;
        let w = match val {
            0 => Wall::Undefined,
            1 => Wall::Open,
            2 => Wall::Wall,
            _ => Wall::Undefined,
        };
        walls.push(w);
    }
    walls
}

// -----------------------------------------------------------------------------
// decode_one_cell => nibble => DecodedCell
// -----------------------------------------------------------------------------
fn decode_one_cell(n: u8) -> DecodedCell {
    println!("Nibble: {:X}", n);
    if n == 0xF {
        println!("Invalid nibble: 0xF");
        return DecodedCell {
            nature: CellNature::Invalid,
            entity: CellEntity::None,
        };
    }
    let nature_bits = (n >> 2) & 0b11;
    let entity_bits = n & 0b11;
    println!("Nature: {:X}, Entity: {:X}", nature_bits, entity_bits);

    let nature = match nature_bits {
        0b00 => CellNature::None,
        0b01 => CellNature::Index,
        0b10 => CellNature::Goal,
        _    => CellNature::Invalid,
    };

    let entity = match entity_bits {
        0b00 => CellEntity::None,
        0b01 => CellEntity::Ally,
        0b10 => CellEntity::Enemy,
        0b11 => CellEntity::Monster,
        _    => CellEntity::None,
    };

    println!("Nature: {:?}, Entity: {:?}", nature, entity);

    DecodedCell { nature, entity }
}

// -----------------------------------------------------------------------------
// decode_cells => 5 octets => 9 nibbles => 9 DecodedCell
// -----------------------------------------------------------------------------
fn decode_cells(bytes: &[u8]) -> Vec<DecodedCell> {
    println!("Bytes: {:?}", bytes);
    let mut bits_40 = 0u64;
    for &b in bytes {
        bits_40 = (bits_40 << 8) | (b as u64);

    }
    println!("Bits: {:X}", bits_40);

    let mut result = Vec::with_capacity(9);
    for i in 0..9 {
        let shift = 36 - 4 * (i);
        let nib = ((bits_40 >> shift) & 0xF) as u8;
        println!("Bits: {:X}, Nibble: {:X}", bits_40 >> shift, nib);
        let cell = decode_one_cell(nib);
        result.push(cell);
    }
    result
}

// -----------------------------------------------------------------------------
// PrettyRadarView => horizontals, verticals, cells
// -----------------------------------------------------------------------------
#[derive(Debug)]
struct PrettyRadarView {
    horizontal_walls: Vec<Wall>,
    vertical_walls: Vec<Wall>,
    cells: Vec<DecodedCell>,
}

// -----------------------------------------------------------------------------
fn interpret_radar_view(h: &[u8], v: &[u8], c: &[u8]) -> PrettyRadarView {
    let horizontal_walls = decode_walls(h);
    let vertical_walls   = decode_walls(v);
    let cells            = decode_cells(c);

    PrettyRadarView {
        horizontal_walls,
        vertical_walls,
        cells,
    }
}

// -----------------------------------------------------------------------------
// ASCII
// -----------------------------------------------------------------------------
fn visualize_radar_ascii(prv: &PrettyRadarView) -> String {
    let mut out = String::new();
    // 4 rangées horizontales
    for row in 0..4 {
        let base = row * 3;
        let walls_slice = &prv.horizontal_walls[base..base+3];
        for &w in walls_slice {
            match w {
                Wall::Undefined => out.push('#'),
                Wall::Open => out.push(' '),
                Wall::Wall => out.push('-'),
            }
            match w {
                Wall::Undefined => out.push('#'),
                Wall::Open => out.push('•'),
                Wall::Wall => out.push('•'),
            }
        }
        out.push_str("\n");
        if row < 3 {
            let start_v = row * 4;
            let v_slice = &prv.vertical_walls[start_v..start_v+4];
            for &vw in v_slice {
                match vw {
                    Wall::Undefined => out.push('#'),
                    Wall::Open => out.push(' '),
                    Wall::Wall => out.push('|'),
                }
                match vw {
                    Wall::Undefined => out.push('#'),
                    _ => {}
                }
            }
            out.push_str("\n");
        }
    }
    out
}

/// Fonction qui affiche les cellules ligne par ligne, style "Undefined, Rien, Undefined".
fn visualize_cells_like_prof(cells: &[DecodedCell]) -> String {
    let mut s = String::new();
    s.push_str("Les cellules (par ligne):\n");
    // 3×3 => 9
    for row in 0..3 {
        let start = row * 3;
        let slice = &cells[start..start+3];
        println!("Slice: {:?}", slice);
        // Ex: "Undefined, Rien, Undefined"
        let mut line_items = Vec::new();
        for &decoded in slice {
            println!("Decoded: {:?}", decoded);
            let cell_str = format_decoded_cell(decoded);
            println!("Cell str: {}", cell_str);
            line_items.push(cell_str);
        }
        s.push_str(&format!(
            "Ligne {} => {}\n",
            row+1,
            line_items.join(", ")
        ));
    }
    s
}

/// Convertit un `DecodedCell` en un string comme "Undefined, Rien" ou "Goal, Ally"
fn format_decoded_cell(c: DecodedCell) -> String {
    println!("DecodedCell nice: {:?}", c);
    let nature_str = match c.nature {
        CellNature::None => "Undefined",
        CellNature::Index => "Index(H)",
        CellNature::Goal => "Goal(G)",
        CellNature::Invalid => "Invalid",
    };
    let entity_str = match c.entity {
        CellEntity::None => "Rien",
        CellEntity::Ally => "Ally(votre position)",
        CellEntity::Enemy => "Enemy",
        CellEntity::Monster => "Monster",
    };

    // Ex: "Undefined, Rien"
    // ou "Goal(G), Ally(votre position)"
    if (nature_str == "Invalid" && entity_str == "Rien") {
        return format!("{}, {}", "Undefined", "Undefined");
    }
    format!("{} + {}", nature_str, entity_str)
}

// -----------------------------------------------------------------------------
// GameStreamHandler
// -----------------------------------------------------------------------------
pub struct GameStreamHandler {
    stream: TcpStream,
    directions: Vec<String>,
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

    fn decide_next_action(&self) -> serde_json::Value {
        let mut rng = rand::rng();
        let default_direction = "Front".to_string();
        let random_direction = self.directions.choose(&mut rng).unwrap_or(&default_direction);
        println!("Decide next action: {}", random_direction);

        json!({ "MoveTo": random_direction })
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
            Ok((h, v, c)) => {
                println!("=== Decoded Raw RadarView ===");
                println!("Horizontals: {:?}", h);
                println!("Verticals:   {:?}", v);
                println!("Cells:       {:?}", c);

                let pretty = interpret_radar_view(&h, &v, &c);
                println!("--- Interpreted RadarView ---");
                println!("Horizontal walls: {:?}", pretty.horizontal_walls);
                println!("Vertical walls:   {:?}", pretty.vertical_walls);
                println!("Cells(decodées)  : {:?}", pretty.cells);

                // (Optionnel) Pour un style "Undefined, Rien, Undefined"
                let cells_str = visualize_cells_like_prof(&pretty.cells);
                println!("{}", cells_str);

                let ascii = visualize_radar_ascii(&pretty);
                println!("--- ASCII Radar ---\n{}", ascii);
                println!("=====================================");
            }
            Err(e) => {
                eprintln!("Erreur lors du décodage du RadarView: {}", e);
            }
        }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        loop {
            let parsed_msg = self.receive_and_parse_message()?;
            if let Some(action_error) = parsed_msg.get("ActionError") {
                println!("ActionError - from server: {:?}", action_error);
                if action_error == "CannotPassThroughWall" {
                    println!("Impossible de passer: mur");
                    continue;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Exiting due to action error: {:?}", action_error),
                    ));
                }
            }
            if let Some(radar_value) = parsed_msg.get("RadarView") {
                if let Some(radar_str) = radar_value.as_str() {
                    self.process_radar_view(radar_str);
                }
            }

            let action = self.decide_next_action();
            self.send_action(&action)?;
        }
    }
}

// -----------------------------------------------------------------------------
// TEST
// -----------------------------------------------------------------------------
#[test]
fn test_radar_ieys() {
    let code = "ieysGjGO8papd/a";
    println!("RadarView code: {:?}", code);

    match decode_radar_view(code) {
        Ok((h, v, c)) => {
            println!("Horizontals = {:?}", h);
            println!("Verticals   = {:?}", v);
            println!("Cells           = {:?}", c);

            let rv = interpret_radar_view(&h, &v, &c);
            println!("Horizontal walls: {:?}", rv.horizontal_walls);
            println!("Vertical   walls: {:?}", rv.vertical_walls);

            // Affiche "Undefined, Rien..." etc.
            let cells_str = visualize_cells_like_prof(&rv.cells);
            println!("{}", cells_str);

            let ascii = visualize_radar_ascii(&rv);
            println!("ASCII:\n{}", ascii);
        }
        Err(e) => println!("Erreur decode_radar_view: {}", e),
    }
}
