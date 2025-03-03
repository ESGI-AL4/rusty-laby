/*!
 * # Gestionnaire de flux de jeu pour Maze
 *
 * Ce module gère la communication entre le client et le serveur de jeu.
 * Il inclut le traitement des messages, la mise à jour de la carte et du joueur,
 * la gestion des défis ainsi que l'affichage graphique via Piston.
 *
 * ## Fonctionnalités
 *
 * - Réception et envoi de messages via TCP.
 * - Décodage et interprétation du RadarView.
 * - Mise à jour de la carte (`MazeMap`) et du joueur (`Player`).
 * - Gestion des challenges via `ChallengeHandler`.
 * - Option d'interface graphique activée avec Piston.
 */

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

// Import de PistonWindow pour l'affichage graphique
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

/// Adresse du serveur de jeu
pub const ADDRESS: &str = "localhost:8778";

/// `GameStreamHandler` gère le flux de communication entre le client et le serveur de jeu.
/// Il contient la carte, le joueur, le gestionnaire de défis et gère l'interface graphique (optionnelle).
pub struct GameStreamHandler {
    stream: TcpStream,
    /// La carte du labyrinthe
    pub map: MazeMap,
    /// Le joueur (position et orientation)
    pub player: Player,
    /// Gestionnaire de défis partagé entre les threads
    pub challenge_handler: Arc<RwLock<ChallengeHandler>>,
    /// Identifiant de l'utilisateur
    pub user_id: usize,
    /// Indique si l'interface graphique (UI) est activée
    pub ui_enabled: bool,
    /// Fenêtre Piston utilisée pour l'affichage graphique (initialisée si `ui_enabled` est vrai)
    pub window: Option<PistonWindow>,
}

impl GameStreamHandler {
    /// Crée une nouvelle instance de `GameStreamHandler`.
    ///
    /// # Arguments
    ///
    /// * `stream` - Flux TCP connecté au serveur.
    /// * `challenge_handler` - Gestionnaire de défis partagé entre les threads.
    /// * `user_id` - Identifiant de l'utilisateur.
    /// * `ui_enabled` - Indique si l'interface graphique est activée.
    ///
    /// # Retour
    ///
    /// Retourne une instance initialisée de `GameStreamHandler`.
    pub fn new(stream: TcpStream, challenge_handler: Arc<RwLock<ChallengeHandler>>, user_id: usize, ui_enabled: bool) -> Self {
        Self {
            stream,
            map: MazeMap::new(),
            // On démarre en position (0,0) orienté vers le Nord.
            player: Player::new(0, 0, Direction::North),
            challenge_handler,
            user_id,
            ui_enabled,
            window: None, // La fenêtre Piston n'est pas encore initialisée.
        }
    }

    /// Initialise la fenêtre Piston pour l'affichage graphique.
    ///
    /// Cette méthode crée une fenêtre de 800x800 pixels avec un framerate maximum de 30 FPS,
    /// et stocke la fenêtre dans `self.window`.
    pub fn init_piston(&mut self) {
        let mut win: PistonWindow = WindowSettings::new("Maze with Piston", [800, 800])
            .exit_on_esc(true)
            .build()
            .unwrap();
        win.set_max_fps(30);

        self.window = Some(win);
    }

    /// Reçoit un message du serveur et le parse en JSON.
    ///
    /// # Retour
    ///
    /// * `Ok(serde_json::Value)` si le message est correctement reçu et parsé.
    /// * Une erreur `io::Error` en cas d'échec.
    fn receive_and_parse_message(&mut self) -> io::Result<serde_json::Value> {
        let msg = network::receive_message(&mut self.stream)?;
        let parsed_msg = json_utils::parse_json(&msg)?;
        Ok(parsed_msg)
    }

    /// Envoie une action au serveur sous forme de JSON.
    ///
    /// # Arguments
    ///
    /// * `action` - L'action à envoyer, formatée en JSON.
    ///
    /// # Retour
    ///
    /// * `Ok(())` en cas de succès, ou une erreur `io::Error` en cas d'échec.
    fn send_action(&mut self, action: &serde_json::Value) -> io::Result<()> {
        let action_request = json!({ "Action": action }).to_string();
        network::send_message(&mut self.stream, &action_request)?;
        Ok(())
    }

    /// Traite le RadarView reçu sous forme de chaîne de caractères.
    ///
    /// Cette fonction décode et interprète le RadarView pour en extraire les informations
    /// sur les murs et les cellules, et retourne un `PrettyRadarView` structuré.
    ///
    /// # Arguments
    ///
    /// * `radar_str` - La chaîne de caractères représentant le RadarView.
    ///
    /// # Retour
    ///
    /// Retourne une instance de `PrettyRadarView` contenant les informations décodées.
    fn process_radar_view(&mut self, radar_str: &str) -> PrettyRadarView {
        match decode_radar_view(radar_str) {
            Ok((h, v, c)) => {
                let pretty = interpret_radar_view(&h, &v, &c);

                // Optionnel : visualisation des cellules et du radar en ASCII
                let _cells_str = visualize_cells_like_prof(&pretty.cells);
                let _ascii = visualize_radar_ascii(&pretty);
                pretty
            }
            Err(e) => {
                eprintln!("Erreur lors du décodage du RadarView: {}", e);
                // Retourne une valeur par défaut en cas d'erreur
                PrettyRadarView {
                    horizontal_walls: vec![],
                    vertical_walls: vec![],
                    cells: vec![],
                }
            }
        }
    }

    /// Boucle principale de traitement des messages du serveur.
    ///
    /// Cette méthode gère :
    /// - La réception et le traitement des messages du serveur.
    /// - Le décodage du RadarView et la mise à jour de la carte et du joueur.
    /// - La prise de décision pour déterminer l'action à effectuer.
    /// - L'envoi de l'action au serveur.
    /// - L'affichage graphique via Piston si l'UI est activée.
    ///
    /// # Retour
    ///
    /// * `Ok(())` si le traitement se termine correctement,
    /// * Ou une erreur `io::Error` en cas de problème.
    pub fn handle(&mut self) -> io::Result<()> {
        let mut win: Option<PistonWindow> = None;
        let mut challenge_count = 0;

        if self.ui_enabled {
            if self.window.is_none() {
                eprintln!("PistonWindow not initialized, call init_piston() first!");
                return Ok(());
            }
            // On récupère la fenêtre localement
            win = Some(self.window.take().unwrap());
        }

        loop {
            let parsed_msg = self.receive_and_parse_message()?;

            // Gestion des erreurs d'action
            if let Some(action_error) = parsed_msg.get("ActionError") {
                if action_error == "CannotPassThroughWall" {
                    // Le joueur ne peut pas passer à travers un mur, on continue la boucle.
                    continue;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Exiting due to action error: {:?}", action_error),
                    ));
                }
            }

            // Traitement du RadarView
            if let Some(radar_value) = parsed_msg.get("RadarView") {
                if let Some(radar_str) = radar_value.as_str() {
                    // 1) Décodage et interprétation du RadarView
                    let pretty = self.process_radar_view(radar_str);

                    // Mise à jour de la carte à partir des informations du RadarView
                    self.map.update_from_radar(&pretty, &mut self.player);

                    // Vérifie si le joueur se trouve sur la sortie
                    let current_cell = pretty.cells[4];
                    if current_cell.nature == bin::radarview::CellNature::Goal {
                        println!("Exit found by player {}", self.user_id);
                        return Ok(());
                    }

                    // Définition des actions possibles
                    let mut moove: Vec<String> = vec![]; // Mouvements vers des cellules non visitées
                    let mut moove_fallback: Vec<String> = vec![]; // Mouvements possibles en fallback
                    if pretty.vertical_walls[6] == Open {
                        moove_fallback.push("Right".to_string());
                        let direction = self.player.direction.relative_to_absolute("Right");
                        if !self.map.is_cell_visited(self.player.clone(), direction) {
                            moove.push("Right".to_string());
                        }
                    }
                    if pretty.vertical_walls[5] == Open {
                        moove_fallback.push("Left".to_string());
                        let direction = self.player.direction.relative_to_absolute("Left");
                        if !self.map.is_cell_visited(self.player.clone(), direction) {
                            moove.push("Left".to_string());
                        }
                    }
                    if pretty.horizontal_walls[4] == Open {
                        moove_fallback.push("Front".to_string());
                        let direction = self.player.direction.relative_to_absolute("Front");
                        if !self.map.is_cell_visited(self.player.clone(), direction) {
                            moove.push("Front".to_string());
                        }
                    }
                    if pretty.horizontal_walls[7] == Open {
                        moove_fallback.push("Back".to_string());
                        let direction = self.player.direction.relative_to_absolute("Back");
                        if !self.map.is_cell_visited(self.player.clone(), direction) {
                            moove.push("Back".to_string());
                        }
                    }

                    let mut back = false;
                    // Si aucune cellule non visitée n'est accessible, revenir en arrière si un chemin existe
                    if moove.is_empty() && !self.player.directions_path.is_empty() {
                        if let Some(move_back) = self.player.directions_path.pop() {
                            back = true;
                            moove.push(move_back.clone());
                            self.player.path.pop();
                        }
                    }
                    // Choix de l'action : privilégier un mouvement vers une cellule non visitée,
                    // sinon utiliser le fallback
                    let action: &String = if !moove.is_empty() {
                        moove.choose(&mut rand::rng()).unwrap()
                    } else {
                        moove_fallback.choose(&mut rand::rng()).unwrap()
                    };
                    let action_json = json!({"MoveTo": action});

                    // Mémorise le mouvement inverse pour revenir en arrière ultérieurement
                    if !back {
                        self.player.directions_path.push(self.player.direction.relative_oposite(action.clone()));
                    }
                    // Envoi de l'action au serveur
                    self.send_action(&action_json)?;

                    // Mise à jour de la position et de l'orientation du joueur
                    self.map.update_player(&mut self.player, &action);

                    // Affichage graphique via Piston si l'UI est activée
                    if self.ui_enabled && win.is_some() {
                        if let Some(win) = win.as_mut() {
                            if let Some(event) = win.next() {
                                if let Some(_r) = event.render_args() {
                                    win.draw_2d(&event, |context, graphics, _device| {
                                        clear([0.0, 0.0, 0.0, 1.0], graphics);
                                        // Dessin de la carte à l'aide de la méthode Piston de MazeMap
                                        self.map.draw_piston(context, graphics, self.player.x, self.player.y, &self.player);
                                    });
                                }
                            }
                        }
                    }

                    // Continuer la boucle de traitement
                    continue;
                }
            }

            // Traitement des challenges et autres messages spécifiques
            self.challenge_handler.write().unwrap().process_message(&parsed_msg, &mut self.stream, &mut challenge_count, self.user_id)?;
        }
    }
}

/// Test de décodage et d'interprétation du RadarView.
///
/// Ce test vérifie que le décodage du RadarView fonctionne et permet d'afficher
/// une visualisation ASCII du résultat.
#[test]
fn test_radar_ieys() {
    let code = "rAeaksua//8a8aa";
    match decode_radar_view(code) {
        Ok((h, v, c)) => {
            let rv = interpret_radar_view(&h, &v, &c);
            let _cells_str = visualize_cells_like_prof(&rv.cells);
            let _ascii = visualize_radar_ascii(&rv);
            // Le test se contente de vérifier que le décodage ne retourne pas d'erreur.
        }
        Err(e) => println!("Erreur decode_radar_view: {}", e),
    }
}
