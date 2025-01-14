use rand::seq::SliceRandom;
use serde_json::json;
use std::io;
use std::io::{BufRead, Write};
use std::net::TcpStream;

mod bin;
use bin::graph_radar::build_graph;

use bin::radarview::{
    decode_radar_view, interpret_radar_view
    ,
};

use crate::bin::ascii_utils::{visualize_cells_like_prof, visualize_radar_ascii};
use crate::bin::{json_utils, network};
use bin::challengehandler::ChallengeHandler;

use crate::bin::map::{Explorer, Map};

pub const ADDRESS: &str = "localhost:8778";

// -----------------------------------------------------------------------------
// GameStreamHandler
// -----------------------------------------------------------------------------
pub struct GameStreamHandler {
    stream: TcpStream,
    directions: Vec<String>,

    /// Notre carte interne (graphe) pour stocker tout ce qu'on découvre
    pub map: Map,

    /// Explorateur gérant la stratégie BFS
    pub explorer: Explorer,
}

impl GameStreamHandler {
    pub fn new(stream: TcpStream) -> Self {
        // On initialise la Map et l'Explorer
        let map = Map::new();
        let explorer = Explorer::new();
        Self {
            stream,
            directions: vec![
                "Front".to_string(),
                "Right".to_string(),
                "Back".to_string(),
                "Left".to_string(),
            ],
            map,
            explorer,
        }
    }

    /*fn decide_next_action(&self) -> serde_json::Value {
        let mut rng = rand::rng();
        let default_direction = "Front".to_string();
        let random_direction = self.directions.choose(&mut rng).unwrap_or(&default_direction);
        println!("Decide next action: {}", random_direction);

        json!({ "MoveTo": random_direction })
    }*/

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

                // 1) Mettre à jour la carte interne (Map)
                self.map.update_with_radarview(&pretty);

                // 2) Dire à l'explorer s'il y a un angle => BFS orienté
                //    Indiquer s'il y a un "Hint(RelativeCompass)" dans ce RadarView
                if let Some(angle) = self.explorer.check_for_compass_hint(&pretty) {
                    println!("Hint: On a détecté un angle = {}", angle);
                    self.explorer.set_angle(angle);
                }
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
                    // 1) Process RadarView => update map
                    self.process_radar_view(radar_str);

                    // 2) Décider la prochaine action (BFS d'exploration)
                    let next_move = self.explorer.decide_next_move(&mut self.map);
                    println!("Decide next action: {}", next_move);

                    // 3) Envoyer l'action
                    let action = json!({ "MoveTo": next_move });
                    self.send_action(&action)?;

                    // 4) Mettre à jour la position du joueur
                    self.map.move_player(&next_move);

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
