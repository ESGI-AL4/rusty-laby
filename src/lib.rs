use std::any::Any;
use serde_json::json;
use std::io;
use std::io::{BufRead, Write};
use std::net::TcpStream;
use rand::seq::{IndexedRandom, SliceRandom};

mod bin;

use bin::radarview::{
    decode_radar_view, interpret_radar_view,
};

use bin::challengehandler::ChallengeHandler;
use crate::bin::{json_utils, network};
use crate::bin::ascii_utils::{visualize_cells_like_prof, visualize_radar_ascii};
use crate::bin::map::{Direction, MazeMap, Player};
use crate::bin::radarview::PrettyRadarView;
use crate::bin::radarview::Wall::Open;

pub const ADDRESS: &str = "localhost:8778";


// -----------------------------------------------------------------------------
// GameStreamHandler
// -----------------------------------------------------------------------------
pub struct GameStreamHandler {
    stream: TcpStream,
    pub map: MazeMap,      // La carte
    pub player: Player,    // Le joueur (position + orientation)
}

impl GameStreamHandler {

    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            map: MazeMap::new(),
            // Imaginons qu'on démarre en (0,0), orienté North
            player: Player::new(0, 0, Direction::North),
        }
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

    fn process_radar_view(&mut self, radar_str: &str) -> PrettyRadarView {
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

                //return pretty;
                pretty

            }
            Err(e) => {
                eprintln!("Erreur lors du décodage du RadarView: {}", e);
                // Retourner une valeur par défaut
                PrettyRadarView {
                    horizontal_walls: vec![],
                    vertical_walls: vec![],
                    cells: vec![],
                }
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
                    let pretty = self.process_radar_view(radar_str);

                    println!("player walls: ");
                    println!("     {:?}", pretty.horizontal_walls[4]);
                    println!("{:?}  /*/   {:?}", pretty.vertical_walls[5], pretty.vertical_walls[6]);
                    println!("     {:?}", pretty.horizontal_walls[7]);

                    // Met à jour la carte en se basant sur ce RadarView
                    self.map.update_from_radar(&pretty, &mut self.player);


                    let mut moove = vec![];

                    // 2) (Option) Décider d'une action (ex: MoveTo "Front")
                    if pretty.vertical_walls[6] == Open {
                        moove.push("Right");
                    }
                    if pretty.vertical_walls[5] == Open {
                        moove.push("Left");
                    }
                    if pretty.horizontal_walls[4] == Open {
                        moove.push("Front");
                    }
                    if pretty.horizontal_walls[7] == Open {
                        moove.push("Back");
                    }

                    println!("moove: {:?}", moove);

                    let action = moove.choose(&mut rand::rng()).unwrap();
                    let action_json = json!({"MoveTo": action});
                    println!("Decide next action: {:?}", action);

                    // Print Player
                    println!("Player: {:?}", self.player);

                    // 2) Envoyer l'action
                    self.send_action(&action_json)?;

                    //update player position and direction
                    self.map.update_player(&mut self.player, &action);

                    // Print Player après l'update
                    println!("Player: {:?}", self.player);

                    // Print Map
                    self.map.display_map(Option::from((self.player.x, self.player.y)));


                    // 3) Attendre que l'utilisateur appuie sur Entrée
                    print!("Appuyez sur Entrée pour avancer...");
                    io::stdout().flush()?; // Assure que le message est affiché avant d'attendre
                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer)?;

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
