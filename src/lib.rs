use serde_json::json;
use std::io;
use std::io::{BufRead, Write};
use std::net::TcpStream;
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;

mod bin;

use bin::radarview::{
    decode_radar_view, interpret_radar_view,
};

use bin::challengehandler::ChallengeHandler;
use crate::bin::{json_utils, network};
use crate::bin::ascii_utils::{visualize_cells_like_prof, visualize_radar_ascii};


pub const ADDRESS: &str = "localhost:8778";


// -----------------------------------------------------------------------------
// GameStreamHandler
// -----------------------------------------------------------------------------
pub struct GameStreamHandler {
    stream: TcpStream,
    directions: Vec<String>,
}

impl GameStreamHandler {
    pub fn new(stream: TcpStream) -> Self {
        // On initialise la Map et l'Explorer
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
        let mut rng = rand::thread_rng();
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

    /// Boucle principale
    pub fn handle(&mut self) -> io::Result<()> {
        let mut challenge_handler = ChallengeHandler::new();
        let mut challenge_count = 0;

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

                    if let Some(radar_view) = parsed_msg.get("RadarView") {
                        println!("RadarView received: {:?}", radar_view);
                    }

                    // 1) Process RadarView => update map
                    self.process_radar_view(radar_str);

                    let action = self.decide_next_action();
                    println!("Decide next action: {}", action);

                    // 2) Envoyer l'action
                    self.send_action(&action)?;

                    // 3) Attendre que l'utilisateur appuie sur Entrée
                    /*print!("Appuyez sur Entrée pour avancer...");
                    io::stdout().flush()?; // Assure que le message est affiché avant d'attendre
                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer)?;*/

                    continue;
                }
            }

            // Gestion des challenges et des secrets
            challenge_handler.process_message(&parsed_msg, &mut self.stream, &mut challenge_count)?;
        }
    }
}

// -----------------------------------------------------------------------------
// TEST
// -----------------------------------------------------------------------------
#[test]
fn test_radar_ieys() {
    let code = "rAeaksua//8a8aa";
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


            // Log the graph structure
            // graph.log_graph();

            // Visualiser le graph en ASCII
            //let graph_ascii = graph.visualize_ascii();
            //println!("--- Graph Visualization ---\n{}", graph_ascii);

        }
        Err(e) => println!("Erreur decode_radar_view: {}", e),
    }
}
