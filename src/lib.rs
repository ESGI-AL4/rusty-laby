use std::io;
use std::io::{Write, BufRead, BufReader};
use std::net::TcpStream;
use rand::seq::IndexedRandom;
use serde_json::{Value, json};

mod bin;
use bin::graph_radar::{build_graph};

use bin::radarview::{
    decode_radar_view, interpret_radar_view, PrettyRadarView, Wall, DecodedCell, CellNature, CellEntity,
};

use bin::challengehandler::ChallengeHandler;

pub const ADDRESS: &str = "localhost:8778";


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
        // Ex: "Undefined, Rien, Undefined"
        let mut line_items = Vec::new();
        for &decoded in slice {
            let cell_str = format_decoded_cell(decoded);
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
    let nature_str = match c.nature {
        CellNature::None => "Rien",
        CellNature::Hint => "Hint(H)",
        CellNature::Goal => "Goal(G)",
        CellNature::Invalid => "Undefined",
    };

    let entity_str = match c.entity {
        CellEntity::None => "Rien",
        CellEntity::Ally => "Ally",
        CellEntity::Enemy => "Enemy",
        CellEntity::Monster => "Monster",
    };

    // Simplify specific cases
    if nature_str == "Undefined" && entity_str == "Rien" {
        return "Undefined".to_string();
    }
    if nature_str == "Rien" && entity_str == "Rien" {
        return "Rien".to_string();
    }

    // Default case
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
        let mut challenge_handler = ChallengeHandler::new();
        let player_name = "Player1";

        loop {
            let parsed_msg = self.receive_and_parse_message()?;

            // Gestion des erreurs d'action
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

            // Gestion des RadarView
            if let Some(radar_value) = parsed_msg.get("RadarView") {
                if let Some(radar_str) = radar_value.as_str() {
                    self.process_radar_view(radar_str);
                    let action = self.decide_next_action();
                    self.send_action(&action)?;
                    continue;
                }
            }

            // Gestion des challenges ou des secrets
            challenge_handler.process_message(&parsed_msg, player_name, &mut self.stream)?;
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
            println!("{:?}", rv.cells);
            let cells_str = visualize_cells_like_prof(&rv.cells);
            println!("{}", cells_str);

            let ascii = visualize_radar_ascii(&rv);
            println!("ASCII:\n{}", ascii);

            // Construire le graph à partir des données interprétées
            let graph = build_graph(&rv.horizontal_walls, &rv.vertical_walls, &rv.cells);

            // Log the graph structure
            // graph.log_graph();

            // Visualiser le graph en ASCII
            //let graph_ascii = graph.visualize_ascii();
            //println!("--- Graph Visualization ---\n{}", graph_ascii);

            // Reconstruire les données depuis le graphe
            let rv_reconstructed = graph.reconstruct_radar_view();
            println!("Reconstructed RadarView: ");
            println!("Horizontal walls: {:?}", rv_reconstructed.horizontal_walls);
            println!("Vertical   walls: {:?}", rv_reconstructed.vertical_walls);
            println!("Cells: {:?}", rv_reconstructed.cells);
            let cells_graph_str = visualize_cells_like_prof(&rv_reconstructed.cells);
            println!("{}", cells_graph_str);
        }
        Err(e) => println!("Erreur decode_radar_view: {}", e),
    }
}
