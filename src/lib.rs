use std::any::Any;
use bin::{challengehandler, direction};
use serde_json::json;
use std::io;
use std::io::{BufRead, Write};
use std::net::TcpStream;
use rand::seq::{IndexedRandom, SliceRandom};
use std::sync::Arc;
use std::sync::RwLock;

pub mod bin;

// On importe piston_window
use piston_window::{
    PistonWindow, WindowSettings, EventLoop, RenderEvent,
    clear, rectangle, Context, G2d,
};

use bin::radarview::{
    decode_radar_view, interpret_radar_view,
};

use bin::challengehandler::ChallengeHandler;
use crate::bin::{json_utils, network};
use crate::bin::ascii_utils::{visualize_cells_like_prof, visualize_radar_ascii};
use crate::bin::map::MazeMap;
use crate::bin::player::Player;
use crate::bin::direction::Direction;
use crate::bin::radarview::PrettyRadarView;
use crate::bin::radarview::Wall::Open;

pub const ADDRESS: &str = "localhost:8778";


// -----------------------------------------------------------------------------
// GameStreamHandler
// -----------------------------------------------------------------------------
pub struct GameStreamHandler {
    stream: TcpStream,
    pub map: MazeMap,      // La carte
    pub player: Player,   // Le joueur (position + orientation)
    pub challenge_handler:Arc<RwLock<ChallengeHandler>>,
    pub user_id: usize,
    pub ui_enabled: bool,
    // Ajout: la fenêtre Piston
    pub window: Option<PistonWindow>,
}

impl GameStreamHandler {

    pub fn new(stream: TcpStream, challenge_handler: Arc<RwLock<ChallengeHandler>>, user_id: usize, ui_enabled: bool) -> Self {
        Self {
            stream,
            map: MazeMap::new(),
            // Imaginons qu'on démarre en (0,0), orienté North
            player: Player::new(0, 0, Direction::North),
            challenge_handler,
            user_id,
            ui_enabled,
            window: None, // pas encore initialisé
        }
    }

    /// On appelle cette fonction dans `main` (après la création).
    /// Ici, on crée la fenêtre PistonWindow, qu’on stocke dans `self.window`.
    pub fn init_piston(&mut self) {
        let mut win: PistonWindow = WindowSettings::new("Maze with Piston", [800, 800])
            .exit_on_esc(true)
            .build()
            .unwrap();
        win.set_max_fps(30);

        self.window = Some(win);
    }

    fn receive_and_parse_message(&mut self) -> io::Result<serde_json::Value> {
        let msg = network::receive_message(&mut self.stream)?;

        let parsed_msg = json_utils::parse_json(&msg)?;
        Ok(parsed_msg)
    }

    fn send_action(&mut self, action: &serde_json::Value) -> io::Result<()> {
        let action_request = json!({ "Action": action }).to_string();
        network::send_message(&mut self.stream, &action_request)?;
        Ok(())
    }

    fn process_radar_view(&mut self, radar_str: &str) -> PrettyRadarView {
        match decode_radar_view(radar_str) {
            Ok((h, v, c)) => {

                let pretty = interpret_radar_view(&h, &v, &c);

                // (Optionnel) Pour un style "Undefined, Rien, Undefined"
                let cells_str = visualize_cells_like_prof(&pretty.cells);

                let ascii = visualize_radar_ascii(&pretty);
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
        // println!("IN THE HANDLE");
        let mut win: Option<PistonWindow> = None;
        let mut challenge_count = 0;
        if self.ui_enabled {
            if self.window.is_none() {
                // on peut paniquer ou le faire nous-même
                eprintln!("PistonWindow not initialized, call init_piston() first!");
                return Ok(());
            }
            win = Some(self.window.take().unwrap()); // on récupère la fenêtre localement
        }

        loop {
            let parsed_msg = self.receive_and_parse_message()?;

            
            // Gestion des erreurs d'action
            if let Some(action_error) = parsed_msg.get("ActionError") {
                //  eprintln!("ActionError - from server: {:?}", action_error);
                if action_error == "CannotPassThroughWall" {
                    // println!("Impossible de passer: mur");
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

                    // 1) Process RadarView
                    let pretty = self.process_radar_view(radar_str);

                    // Met à jour la carte en se basant sur ce RadarView
                    self.map.update_from_radar(&pretty, &mut self.player);


                    //Check if on the exit
                    let current_cell = pretty.cells[4];
                    if current_cell.nature == bin::radarview::CellNature::Goal {
                        println!("Exit found by player {}", self.user_id);
                        return Ok(());
                    }

                    let mut moove: Vec<String> = vec![]; // Possible moves to opened non visited cells
                    let mut moove_fallback: Vec<String> = vec![]; // All possible moves to opened cells if no non visited cells and path is empty
                    // 2) (Option) Décider d'une action (ex: MoveTo "Front")
                    if pretty.vertical_walls[6] == Open {
                        moove_fallback.push("Right".to_string());
                        let direction = self.player.direction.relative_to_absolute("Right");
                        if self.map.is_cell_visited(self.player.clone(), direction) == false {
                            moove.push("Right".to_string());
                        }
                    }
                    if pretty.vertical_walls[5] == Open {
                        moove_fallback.push("Left".to_string());
                        let direction = self.player.direction.relative_to_absolute("Left");
                        if self.map.is_cell_visited(self.player.clone(), direction) == false {
                            moove.push("Left".to_string());
                        }
                    }
                    if pretty.horizontal_walls[4] == Open {
                        moove_fallback.push("Front".to_string());
                        let direction = self.player.direction.relative_to_absolute("Front");
                        if self.map.is_cell_visited(self.player.clone(), direction) == false {
                            moove.push("Front".to_string());
                        }
                    }
                    if pretty.horizontal_walls[7] == Open {
                        moove_fallback.push("Back".to_string());
                        let direction: Direction = self.player.direction.relative_to_absolute( "Back");
                        if self.map.is_cell_visited(self.player.clone(), direction) == false {
                            moove.push("Back".to_string()); 
                        }
                    }

                    let mut back = false;
                    // If we can't move to a non visited cell, we add the move to go back on path (If path is not empty)
                    if moove.is_empty() && !self.player.directions_path.is_empty() {
                        if let Some(move_back) = self.player.directions_path.pop() {
                            back = true;
                            moove.push(move_back.clone());
                            self.player.path.pop();
                        }
                    }
                    // use classic moves or fallback if no classic moves
                    let action: &String = if !moove.is_empty() { moove.choose(&mut rand::rng()).unwrap() } else { moove_fallback.choose(&mut rand::rng()).unwrap() };
                    let action_json = json!({"MoveTo": action});

                    // 2) Envoyer l'action
                    if !back {
                        self.player.directions_path.push(self.player.direction.relative_oposite(action.clone()));
                    }
                    self.send_action(&action_json)?;

                    //update player position and direction
                    self.map.update_player(&mut self.player, &action);

                    if self.ui_enabled && win.is_some() {
                        if let Some(win) = win.as_mut() {
                            if let Some(event) = win.next() {
                                // 1) dessiner
                                if let Some(_r) = event.render_args() {
                                    // on dessine
                                    win.draw_2d(&event, |context, graphics, _device| {
                                        clear([0.0, 0.0, 0.0, 1.0], graphics);

                                        // On appelle la fonction piston de la map
                                        self.map.draw_piston(context, graphics, self.player.x, self.player.y, &self.player);
                                    });
                                }
                            }
                        }
                    }


                    // 3) Attendre que l'utilisateur appuie sur Entrée
                    // print!("Appuyez sur Entrée pour avancer...");
                    // io::stdout().flush()?; // Assure que le message est affiché avant d'attendre
                    // let mut buffer = String::new();
                    // io::stdin().read_line(&mut buffer)?;


                    continue;
                }
            }

            // Gestion des challenges et des secrets
            self.challenge_handler.write().unwrap().process_message(&parsed_msg, &mut self.stream, &mut challenge_count, self.user_id)?;
        }
    }
}

// -----------------------------------------------------------------------------
// TEST
// -----------------------------------------------------------------------------
#[test]
fn test_radar_ieys() {
    let code = "rAeaksua//8a8aa";
    // println!("RadarView code: {:?}", code);

    match decode_radar_view(code) {
        Ok((h, v, c)) => {
            // println!("Horizontals = {:?}", h);
            // println!("Verticals   = {:?}", v);
            // println!("Cells           = {:?}", c);

            let rv = interpret_radar_view(&h, &v, &c);
            // println!("Horizontal walls: {:?}", rv.horizontal_walls);
            // println!("Vertical   walls: {:?}", rv.vertical_walls);

            // Affiche "Undefined, Rien..." etc.
            // println!("{:?}", rv.cells);
            let cells_str = visualize_cells_like_prof(&rv.cells);
            // println!("{}", cells_str);

            let ascii = visualize_radar_ascii(&rv);
            // println!("ASCII:\n{}", ascii);


            // Log the graph structure
            // graph.log_graph();

            // Visualiser le graph en ASCII
            //let graph_ascii = graph.visualize_ascii();
            //println!("--- Graph Visualization ---\n{}", graph_ascii);

        }
        Err(e) => println!("Erreur decode_radar_view: {}", e),
    }
}
