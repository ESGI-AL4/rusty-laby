use std::collections::{HashMap, HashSet, VecDeque};
use std::f32::consts::PI;

use rand::seq::SliceRandom;        // <-- Remplace IndexedRandom
use crate::bin::graph_radar::Graph;
use crate::bin::radarview::{CellNature, DecodedCell, PrettyRadarView, Wall};
use crate::bin::radarview::Wall as GWall; // On renomme `Wall` en `GWall` pour le graphe

/// `Map` : Stocke la carte sous forme de graphe, la position du joueur, etc.
pub struct Map {
    pub graph: Graph,
    /// Permet de savoir quelle position (x,y) correspond à tel node_id
    /// et inversement.
    pub pos_to_node: HashMap<(i32, i32), usize>,
    pub node_to_pos: HashMap<usize, (i32, i32)>,

    /// Position actuelle du joueur
    pub position: (i32, i32),
    /// Cases visitées
    pub visited: HashSet<(i32, i32)>,
    /// Chemin parcouru par le joueur
    pub path: Vec<(i32, i32)>,
}

impl Map {
    /// Crée une carte vide (joueur en 0,0)
    pub fn new() -> Self {
        let mut visited = HashSet::new();
        visited.insert((0, 0));

        Map {
            graph: Graph::new(),
            pos_to_node: HashMap::new(),
            node_to_pos: HashMap::new(),
            position: (0, 0),
            visited,
            path: vec![(0, 0)],
        }
    }

    /// Ajoute ou récupère un node_id pour la position (x,y).
    fn get_or_create_node(&mut self, x: i32, y: i32, cell: DecodedCell) -> usize {
        if let Some(&nid) = self.pos_to_node.get(&(x, y)) {
            // On peut mettre à jour la nature si on découvre un "Goal" par ex.
            if let Some(node) = self.graph.nodes.get_mut(&nid) {
                if cell.nature == CellNature::Goal {
                    node.cell.nature = CellNature::Goal;
                }
            }
            return nid;
        }

        let new_id = self.graph.nodes.len();
        self.graph.add_node(new_id, cell);
        self.pos_to_node.insert((x, y), new_id);
        self.node_to_pos.insert(new_id, (x, y));
        new_id
    }

    /// Retourne le node_id correspondant à (x,y), si connu
    fn get_node_id(&self, x: i32, y: i32) -> Option<usize> {
        self.pos_to_node.get(&(x, y)).copied()
    }

    /// Met à jour la carte à partir d'un RadarView
    pub fn update_with_radarview(&mut self, rv: &PrettyRadarView) {
        let (px, py) = self.position;

        // Cellule centrale
        let center_cell = rv.cells[4];
        let _center_id = self.get_or_create_node(px, py, center_cell); // prefixé `_` si non utilisé

        // Marque visitée
        self.visited.insert((px, py));

        // Parcourt les 9 cellules du radar (3x3)
        for row in 0..3 {
            for col in 0..3 {
                let idx = row * 3 + col;
                let dx = col as i32 - 1;
                let dy = row as i32 - 1;
                let cx = px + dx;
                let cy = py + dy;

                let dec_cell = rv.cells[idx];
                let this_id = self.get_or_create_node(cx, cy, dec_cell);

                // MUR HAUT
                if row > 0 {
                    if let Some(&ww) = rv.horizontal_walls.get((row - 1) * 3 + col) {
                        if ww == Wall::Open {
                            let up_pos = (cx, cy - 1);
                            if let Some(up_id) = self.get_node_id(up_pos.0, up_pos.1) {
                                self.graph.add_edge(this_id, up_id, GWall::Open);
                            }
                        }
                    }
                }

                // MUR BAS
                if row < 2 {
                    if let Some(&ww) = rv.horizontal_walls.get(row * 3 + col) {
                        if ww == Wall::Open {
                            let down_pos = (cx, cy + 1);
                            if let Some(down_id) = self.get_node_id(down_pos.0, down_pos.1) {
                                self.graph.add_edge(this_id, down_id, GWall::Open);
                            }
                        }
                    }
                }

                // MUR GAUCHE
                if col > 0 {
                    if let Some(&ww) = rv.vertical_walls.get(row * 4 + (col - 1)) {
                        if ww == Wall::Open {
                            let left_pos = (cx - 1, cy);
                            if let Some(left_id) = self.get_node_id(left_pos.0, left_pos.1) {
                                self.graph.add_edge(this_id, left_id, GWall::Open);
                            }
                        }
                    }
                }

                // MUR DROIT
                if col < 2 {
                    if let Some(&ww) = rv.vertical_walls.get(row * 4 + col) {
                        if ww == Wall::Open {
                            let right_pos = (cx + 1, cy);
                            if let Some(right_id) = self.get_node_id(right_pos.0, right_pos.1) {
                                self.graph.add_edge(this_id, right_id, GWall::Open);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Déplace la position du joueur selon la direction
    pub fn move_player(&mut self, direction: &str) {
        let (x, y) = self.position;
        let new_pos = match direction {
            "Front" => (x, y + 1),
            "Back" => (x, y - 1),
            "Left" => (x - 1, y),
            "Right" => (x + 1, y),
            _ => (x, y),
        };
        self.position = new_pos;
        self.visited.insert(new_pos);
        self.path.push(new_pos);
    }

    /// Visualise la carte
    pub fn visualize_map(&self) -> String {
        let mut out = self.graph.visualize_ascii();
        out.push_str("\n--- Player Path ---\n");
        for (i, pos) in self.path.iter().enumerate() {
            out.push_str(&format!("Step {}: {:?}\n", i + 1, pos));
        }
        out
    }
}

// ============================================================================

/// `Explorer` : gère la logique BFS + orientation si on a un angle
pub struct Explorer {
    /// angle d'orientation si on a trouvé un "RelativeCompass"
    /// None => on fait un BFS normal
    /// Some(angle) => BFS orienté
    pub compass_angle: Option<f32>,
}

impl Explorer {
    pub fn new() -> Self {
        Explorer { compass_angle: None }
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.compass_angle = Some(angle);
    }

    /// Parcourt le RadarView pour voir si on détecte un "Hint(RelativeCompass)"
    /// => simpliste : si on voit un CellNature::Hint, on suppose un angle.
    ///   Dans la vraie vie, vous recevrez l'angle en `Hint { RelativeCompass { angle } }`
    pub fn check_for_compass_hint(&self, _rv: &PrettyRadarView) -> Option<f32> {
        // À implémenter si on veut détecter l'angle depuis le JSON
        None
    }

    /// Décide le prochain mouvement : BFS standard ou BFS orienté
    pub fn decide_next_move(&self, map: &mut Map) -> String {
        // 1) Chercher un Goal
        if let Some(goal_pos) = self.find_any_goal(map) {
            if let Some(next_dir) = self.bfs_next_step(map, goal_pos.0, goal_pos.1) {
                return next_dir;
            }
        }
        // 2) Sinon BFS vers une case non visitée
        /*if let Some(next_dir) = self.bfs_exploration(map) {
            println!("BFS Exploration");
            return next_dir;
        }*/
        // 3) Aléatoire
        self.random_direction()
    }

    /// Cherche un node "Goal" dans graph
    fn find_any_goal(&self, map: &Map) -> Option<(i32, i32)> {
        for (nid, node) in &map.graph.nodes {
            if node.cell.nature == CellNature::Goal {
                return Some(map.node_to_pos[&nid]);
            }
        }
        None
    }

    /// BFS vers la *prochaine case non visitée* la plus proche
    fn bfs_exploration(&self, map: &Map) -> Option<String> {
        let start_id = map.get_node_id(map.position.0, map.position.1)?;
        let mut queue = VecDeque::new();
        let mut visited_nids = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(start_id);
        visited_nids.insert(start_id);

        while let Some(curr) = queue.pop_front() {
            if let Some(node) = map.graph.nodes.get(&curr) {
                for (&n_id, &ww) in &node.neighbors {
                    if ww == GWall::Open && visited_nids.insert(n_id) {
                        parent.insert(n_id, curr);

                        let (nx, ny) = map.node_to_pos[&n_id];
                        // Si on trouve une cellule non visitée, on s'arrête
                        if !map.visited.contains(&(nx, ny)) {
                            let path_ids = reconstruct_path(n_id, &parent);
                            if path_ids.len() >= 2 {
                                let first_step_id = path_ids[1];
                                let dir = direction_from(map.position, map.node_to_pos[&first_step_id]);
                                return Some(dir);
                            }
                        }
                        queue.push_back(n_id);
                    }
                }
            }
        }
        None
    }

    /// BFS "classique" vers (tx, ty), renvoie la première direction
    fn bfs_next_step(&self, map: &Map, tx: i32, ty: i32) -> Option<String> {
        let start_id = map.get_node_id(map.position.0, map.position.1)?;
        let goal_id = map.get_node_id(tx, ty)?;

        let mut queue = VecDeque::new();
        let mut visited_nids = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(start_id);
        visited_nids.insert(start_id);

        while let Some(curr) = queue.pop_front() {
            if curr == goal_id {
                // Reconstitution du chemin
                let path_ids = reconstruct_path(curr, &parent);
                if path_ids.len() >= 2 {
                    let first_step_id = path_ids[1];
                    let dir = direction_from(map.position, map.node_to_pos[&first_step_id]);
                    return Some(dir);
                } else {
                    return None;
                }
            }
            if let Some(node) = map.graph.nodes.get(&curr) {
                // BFS orienté
                let mut neighs: Vec<usize> = node
                    .neighbors
                    .iter()
                    .filter_map(|(&nid, &ww)| {
                        if ww == GWall::Open {
                            Some(nid)
                        } else {
                            None
                        }
                    })
                    .collect();

                // Si on a un angle, on trie
                if let Some(angle) = self.compass_angle {
                    neighs.sort_by_key(|&nid| {
                        let (nx, ny) = map.node_to_pos[&nid];
                        let angle_diff = angle_distance(angle, direction_angle(map.position, (nx, ny)));
                        (angle_diff * 1000.0) as u32
                    });
                }

                for &nid in &neighs {
                    if visited_nids.insert(nid) {
                        parent.insert(nid, curr);
                        queue.push_back(nid);
                    }
                }
            }
        }
        None
    }

    fn random_direction(&self) -> String {
        let directions = ["Front", "Right", "Back", "Left"];
        let mut rng = rand::thread_rng(); // <-- thread_rng()
        directions.choose(&mut rng).unwrap_or(&"Front").to_string()
    }
}

// Reconstruit le chemin BFS (liste de node_id)
fn reconstruct_path(mut curr: usize, parent: &HashMap<usize, usize>) -> Vec<usize> {
    let mut path = vec![curr];
    while let Some(&p) = parent.get(&curr) {
        curr = p;
        path.push(curr);
    }
    path.reverse();
    path
}

/// Calcule la direction (Front/Right/Back/Left) entre 2 positions
fn direction_from(from: (i32, i32), to: (i32, i32)) -> String {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    match (dx, dy) {
        (0, 1) => "Front".to_string(),
        (0, -1) => "Back".to_string(),
        (1, 0) => "Right".to_string(),
        (-1, 0) => "Left".to_string(),
        _ => "Front".to_string(),
    }
}

/// Calcule l'angle en radians
fn direction_angle(from: (i32, i32), to: (i32, i32)) -> f32 {
    let dx = (to.0 - from.0) as f32;
    let dy = (to.1 - from.1) as f32;
    dy.atan2(dx) // 0 rad = direction +x
}

/// Distance angulaire entre 2 angles
fn angle_distance(a: f32, b: f32) -> f32 {
    let mut diff = (a - b).abs();
    while diff > PI {
        diff -= 2.0 * PI;
    }
    diff.abs()
}
