use crate::bin::graph_radar::{Graph, Node};
use crate::bin::radarview::{PrettyRadarView, Wall, DecodedCell};
use std::collections::{HashMap, HashSet};

/// Structure représentant la carte du jeu et l'état du joueur
pub struct Map {
    pub graph: Graph,                     // Graphe des cellules explorées
    pub position: (i32, i32),             // Position actuelle du joueur (x, y)
    pub visited: HashSet<(i32, i32)>,     // Cellules déjà visitées
    pub path: Vec<(i32, i32)>,            // Chemin parcouru par le joueur
}

impl Map {
    /// Crée une nouvelle carte
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            position: (0, 0), // Position initiale du joueur
            visited: HashSet::new(),
            path: vec![(0, 0)], // Le joueur commence à (0, 0)
        }
    }

    /// Met à jour la carte à partir d'un RadarView
    pub fn update_with_radarview(&mut self, radar_view: PrettyRadarView) {
        // Ajoutez la cellule actuelle au graphe si non déjà enregistrée
        if !self.visited.contains(&self.position) {
            let cell_id = self.graph.nodes.len();
            let current_cell = radar_view.cells[4]; // Cellule centrale correspond au joueur
            self.graph.add_node(cell_id, current_cell);

            // Marquer comme visité
            self.visited.insert(self.position);
        }

        // Ajouter les voisins visibles à partir du RadarView
        for (direction, wall) in radar_view.visible_neighbors() {
            let neighbor_position = self.calculate_neighbor_position(&direction);
            if !self.visited.contains(&neighbor_position) {
                self.graph.add_edge(self.position, neighbor_position, wall);
            }
        }
    }


    /// Met à jour la position actuelle et le chemin du joueur
    pub fn move_player(&mut self, direction: &str) {
        let new_position = self.calculate_neighbor_position(direction);
        self.position = new_position;

        // Ajouter la nouvelle position au chemin parcouru
        self.path.push(new_position);
    }

    /// Calcule la position relative d'un voisin en fonction de la direction
    fn calculate_neighbor_position(&self, direction: &str) -> (i32, i32) {
        match direction {
            "Front" => (self.position.0, self.position.1 + 1),
            "Right" => (self.position.0 + 1, self.position.1),
            "Back" => (self.position.0, self.position.1 - 1),
            "Left" => (self.position.0 - 1, self.position.1),
            _ => self.position,
        }
    }

    /// Visualise la carte et le chemin parcouru sous forme ASCII
    pub fn visualize_map(&self) -> String {
        // Appeler la visualisation ASCII du graphe
        let mut ascii_map = self.graph.visualize_ascii();

        // Ajouter le chemin parcouru
        ascii_map.push_str("\n--- Player Path ---\n");
        for (i, pos) in self.path.iter().enumerate() {
            ascii_map.push_str(&format!("Step {}: {:?}\n", i + 1, pos));
        }

        ascii_map
    }

    /// Vérifie si une cellule a déjà été visitée
    pub fn is_visited(&self, position: (i32, i32)) -> bool {
        self.visited.contains(&position)
    }
}

