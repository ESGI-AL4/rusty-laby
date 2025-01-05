use std::io;
use std::io::{Write, BufRead, BufReader};
use std::net::TcpStream;
use rand::seq::IndexedRandom;
use serde_json::{Value, json};

pub const ADDRESS: &str = "localhost:8778";

// -----------------------------------------------------------------------------
// Imports pour le custom base64
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

    pub fn subscribe_player(
        &mut self,
        player_name: &str,
        registration_token: &str,
        mut stream: TcpStream,
    ) -> std::io::Result<String> {
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
            println!(
                "Server - parsed subscription response: {}",
                parsed_msg["SubscribePlayerResult"]
            );

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

/// Décode la chaîne `s` en bytes via l'alphabet custom.
fn decode_custom_b64(s: &str) -> Result<Vec<u8>, DecodeError> {
    let alphabet = match Alphabet::new(CUSTOM_ALPHABET_STR) {
        Ok(a) => a,
        Err(_parse_err) => {
            return Err(DecodeError::InvalidByte(0, b'?'));
        }
    };

    let config = GeneralPurposeConfig::new()
        .with_decode_padding_mode(DecodePaddingMode::Indifferent)
        .with_decode_allow_trailing_bits(true);

    let engine = GeneralPurpose::new(&alphabet, config);

    engine.decode(s)
}

/// Décode un RadarView en renvoyant 3 blocs (h,v,c)
fn decode_radar_view(
    radar_b64: &str
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let bytes = decode_custom_b64(radar_b64)?;

    // On suppose 11 octets (3 + 3 + 5)
    if bytes.len() < 11 {
        return Err(format!("RadarView: {} octets reçus, on s’attend à 11", bytes.len()).into());
    }

    let horizontals = bytes[0..3].to_vec();
    let verticals   = bytes[3..6].to_vec();
    let cells       = bytes[6..11].to_vec();

    Ok((horizontals, verticals, cells))
}

// -----------------------------------------------------------------------------
// Interprétation + ASCII
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
enum Wall {
    Undefined,
    Open,
    Wall,
}

#[derive(Debug)]
struct Cell {
    item_type: u8,
}

#[derive(Debug)]
struct PrettyRadarView {
    horizontal_walls: Vec<Wall>,
    vertical_walls: Vec<Wall>,
    cells: Vec<Cell>,
}

// 12 murs horizontaux => 3 octets => 2 bits par mur
fn decode_walls(bytes: &[u8]) -> Vec<Wall> {
    let mut walls = Vec::new();
    for i in 0..12 {
        let byte_index = i / 4;
        let bit_index = (i % 4) * 2;
        let val = (bytes[byte_index] >> bit_index) & 0b11;
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

// 9 cellules => 5 octets => 4 bits par cellule
fn decode_cells(bytes: &[u8]) -> Vec<Cell> {
    let mut cells = Vec::new();
    for i in 0..9 {
        let byte_index = i / 2;
        let nibble_index = (i % 2) * 4;
        let item_type = (bytes[byte_index] >> nibble_index) & 0xF;
        cells.push(Cell { item_type });
    }
    cells
}

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

/// Crée un ASCII 7×7 environ pour dessiner un radar 3×3
fn visualize_radar_ascii(prv: &PrettyRadarView) -> String {
    // On va dessiner un petit tableau 7×7 (ou 6×6) : 4 lignes horizontales
    //   (car 3 "cases" + 1) et 4 lignes verticales.
    // La logique est similaire à votre code existant, mais adaptée à nos "Wall".
    //
    // Indice sur l'indexation :
    //   horizontal_walls : 12 => 4 rangées × 3 segments
    //   vertical_walls   : 12 => 4 colonnes × 3 segments
    //   cells            : 9 => 3×3
    //
    // Pour faire simple, on se base sur l'exemple "visualize_radar" que vous aviez,
    // mais on l'adapte à `PrettyRadarView`.

    let mut output = String::new();
    // --- Première ligne (3 segments horizontaux) ---
    //   On prend prv.horizontal_walls[0..3]
    //   0 => segment #1, 1 => #2, 2 => #3
    output.push_str("   ");
    // Intersections + segments
    for i in 0..3 {
        output.push(symbol_h(prv.horizontal_walls[i]));
        output.push('•');
    }
    output.push('\n');

    // On dessine 3 "lignes" de radar
    for row in 0..3 {
        // (1) ligne verticale => 4 segments (vertical_walls)
        //   Les segments verticaux pour la "row" :
        //   indexes : row*4..(row*4 + 4)
        let start_v = row * 4;
        output.push_str("   ");
        for col in 0..4 {
            let wall = prv.vertical_walls[start_v + col];
            output.push(symbol_v(wall));
        }
        output.push('\n');

        // (2) ligne horizontale => 3 segments
        //   indexes : (row+1)*3..(row+1)*3+3
        if row < 2 {
            let start_h = (row + 1) * 3;
            output.push_str("   ");
            for i in 0..3 {
                output.push(symbol_h(prv.horizontal_walls[start_h + i]));
                output.push('•');
            }
            output.push('\n');
        }
    }

    // Note : c'est un exemple simplifié, vous pouvez ajuster la mise en page.
    // Vous pouvez aussi imbriquer la loop "row" et "col" pour afficher
    // un vrai mini-grille 7×7 avec intersections.

    output
}

fn symbol_h(wall: Wall) -> char {
    match wall {
        Wall::Undefined => '•',
        Wall::Open => '-',
        Wall::Wall => '━',
    }
}
fn symbol_v(wall: Wall) -> char {
    match wall {
        Wall::Undefined => '|',
        Wall::Open => ' ',
        Wall::Wall => '|',
    }
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

    /// Décide la prochaine action (aléatoire)
    fn decide_next_action(&self) -> serde_json::Value {
        let mut rng = rand::rng();
        let default_direction = "Front".to_string();
        let random_direction = self.directions.choose(&mut rng).unwrap_or(&default_direction);
        println!("Decide next action: {}", random_direction);

        json!({ "MoveTo": random_direction })
    }

    /// Lit un message du serveur (JSON)
    fn receive_and_parse_message(&mut self) -> io::Result<serde_json::Value> {
        let msg = network::receive_message(&mut self.stream)?;
        println!("Server - received message: {}", msg);

        let parsed_msg = json_utils::parse_json(&msg)?;
        Ok(parsed_msg)
    }

    /// Envoie une action au serveur
    fn send_action(&mut self, action: &serde_json::Value) -> io::Result<()> {
        let action_request = json!({ "Action": action }).to_string();
        println!("Client - Action to server: {}", action_request);
        network::send_message(&mut self.stream, &action_request)?;
        Ok(())
    }

    /// Gère le contenu du RadarView :
    ///   - decode (h, v, c)
    ///   - interpret -> PrettyRadarView
    ///   - puis ASCII
    fn process_radar_view(&mut self, radar_str: &str) {
        // (1) Décodage simple : (horizontals, verticals, cells)
        match decode_radar_view(radar_str) {
            Ok((h, v, c)) => {
                println!("=== Decoded Raw RadarView ===");
                println!("Horizontals (3 octets): {:?}", h);
                println!("Verticals   (3 octets): {:?}", v);
                println!("Cells       (5 octets): {:?}", c);

                // (2) Interprétation
                let pretty = interpret_radar_view(&h, &v, &c);
                println!("--- Interpreted RadarView ---");
                println!("Horizontal walls: {:?}", pretty.horizontal_walls);
                println!("Vertical walls:   {:?}", pretty.vertical_walls);
                println!("Cells:            {:?}", pretty.cells);

                // (3) ASCII
                let ascii = visualize_radar_ascii(&pretty);
                println!("--- ASCII Radar ---\n{}", ascii);
                println!("=====================================");
            }
            Err(e) => {
                eprintln!("Erreur lors du décodage du RadarView: {}", e);
            }
        }
    }

    /// Boucle principale
    pub fn handle(&mut self) -> io::Result<()> {
        loop {
            // 1) Lire un message depuis le serveur
            let parsed_msg = self.receive_and_parse_message()?;

            // 2) Si on a un ActionError
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

            // 3) Si on a un RadarView => on l'interprète + ASCII
            if let Some(radar_value) = parsed_msg.get("RadarView") {
                if let Some(radar_str) = radar_value.as_str() {
                    self.process_radar_view(radar_str);
                }
            }

            // 4) Décider et envoyer une action
            let action = self.decide_next_action();
            self.send_action(&action)?;
        }
    }
}


#[test]
fn test_radar_wzvjMPzbdWaaaaa() {
    let code = "ieysGjGO8papd/a";
    println!("RadarView code: {:?}", code);

    // 1) Décodage brut : 3 blocs (horizontals, verticals, cells)
    match decode_radar_view(code) {
        Ok((h, v, c)) => {
            println!("--- Decoded Raw RadarView ---");
            println!("Horizontals: {:?}", h);
            println!("Verticals:   {:?}", v);
            println!("Cells:       {:?}", c);

            // 2) Interprétation
            let prv = interpret_radar_view(&h, &v, &c);
            println!("--- Interpreted RadarView ---");
            println!("Horizontal walls: {:?}", prv.horizontal_walls);
            println!("Vertical walls:   {:?}", prv.vertical_walls);
            println!("Cells:            {:?}", prv.cells);

            // 3) ASCII
            let ascii = visualize_radar_ascii(&prv);
            println!("--- ASCII Radar ---\n{}", ascii);
            println!("=====================================");
        }
        Err(e) => {
            eprintln!("Erreur lors du décodage: {}", e);
            // Optionnel : on peut faire un `panic!()` si on considère que c'est un test "fail"
        }
    }
}
