use std::collections::HashMap;

use crate::bin::radarview::{DecodedCell, PrettyRadarView, Wall};

/// Représente un nœud (cellule) dans le graphe.
/// `neighbors` associe un autre node_id à un type de mur `Wall`.
#[derive(Debug)]
pub struct Node {
    pub(crate) id: usize,
    pub(crate) cell: DecodedCell,
    /// Exemple : `neighbors.insert(other_node_id, Wall::Open);`
    /// Ainsi, on sait qu'entre `id` et `other_node_id`, on a un mur Open (ou Wall, etc.)
    pub(crate) neighbors: HashMap<usize, Wall>,
}

/// Représente le graphe complet découvert jusqu'à présent.
#[derive(Debug)]
pub struct Graph {
    /// Les nœuds identifiés par leur `id`.
    pub(crate) nodes: HashMap<usize, Node>,

    /// Les murs entre deux nœuds (stockés sous forme de paires `(id_from, id_to)`)
    /// pour être capable de visualiser ou d'y accéder autrement.
    pub(crate) walls: HashMap<(usize, usize), Wall>,
}

impl Graph {
    /// Crée un graphe vide.
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            walls: HashMap::new(),
        }
    }

    /// Ajoute un nœud au graphe (s'il n'existe pas déjà).
    /// `id` est l'identifiant unique du nœud, `cell` est son contenu décodé (Nature + Entity).
    pub fn add_node(&mut self, id: usize, cell: DecodedCell) {
        if !self.nodes.contains_key(&id) {
            self.nodes.insert(
                id,
                Node {
                    id,
                    cell,
                    neighbors: HashMap::new(),
                },
            );
        }
    }

    /// Ajoute une arête (edge) "bidirectionnelle" entre `from` et `to`,
    /// avec un type de mur `wall`.
    /// - Si `wall == Wall::Open`, cela signifie que le déplacement est possible.
    /// - Si `Wall::Wall`, c’est bloqué, etc.
    pub fn add_edge(&mut self, from: usize, to: usize, wall: Wall) {
        // On enregistre l'info dans `walls`
        self.walls.insert((from, to), wall);
        self.walls.insert((to, from), wall);

        // On met à jour `neighbors` si les nœuds existent déjà
        if let Some(node_from) = self.nodes.get_mut(&from) {
            node_from.neighbors.insert(to, wall);
        }
        if let Some(node_to) = self.nodes.get_mut(&to) {
            node_to.neighbors.insert(from, wall);
        }
    }

    /// Exemple minimal de visualisation ASCII (facultatif).
    /// Vous pouvez l'adapter pour réellement tracer un grand labyrinthe
    /// si `nodes` et `walls` correspondent à une grille plus large.
    pub fn visualize_ascii(&self) -> String {
        // Ici, on fournit juste un message placeholder
        // car le véritable affichage dépend de la taille du graphe / labyrinthe.
        "ASCII visualization not fully implemented".to_string()
    }

    /// Affiche dans la console la liste des nœuds et des murs.
    pub fn log_graph(&self) {
        println!("=== Graph Nodes ===");
        for (id, node) in &self.nodes {
            println!(
                "Node {}: {:?}, Neighbors: {:?}",
                id,
                node.cell,
                node.neighbors.keys()
            );
        }

        println!("\n=== Graph Walls ===");
        for ((from, to), wall) in &self.walls {
            println!("Wall between {} and {}: {:?}", from, to, wall);
        }
        println!("===================\n");
    }

    /// Si vous voulez reconstruire un RadarView 3x3 depuis ce graphe,
    /// implémentez la logique dans cette fonction (optionnelle).
    pub fn reconstruct_radar_view(&self) -> PrettyRadarView {
        unimplemented!("Optionnel: reconstruire un RadarView 3x3 depuis le graphe");
    }
}

/// Construit un graphe depuis un `RadarView` 3x3 "statique" (ex. pour des tests).
/// - `horizontals` : liste de 12 murs horizontaux (2 bits chacun),
/// - `verticals` : liste de 12 murs verticaux,
/// - `cells` : 9 `DecodedCell`.
///
/// Dans votre code, vous n'êtes pas obligé d'appeler ceci si vous faites
/// l'intégration progressive dans une structure plus vaste.
pub fn build_graph(horizontals: &[Wall], verticals: &[Wall], cells: &[DecodedCell]) -> Graph {
    let mut graph = Graph::new();

    println!("=== Building Graph ===");
    println!("Horizontals: {:?}", horizontals);
    println!("Verticals: {:?}", verticals);
    println!("Cells: {:?}", cells);

    // 1. Ajouter chaque cellule comme un nœud
    for (i, cell) in cells.iter().enumerate() {
        graph.add_node(i, *cell);
    }

    // 2. Ajouter les liens horizontaux
    //   - On a 4 rangées (row in 0..4) de 3 "morceaux" => 12 total
    //   - L'identifiant du nœud (row*3 + col)
    for row in 0..4 {
        for col in 0..3 {
            let id = row * 3 + col;  // nœud courant
            let next_id = id + 1;    // nœud à droite
            let wall_index = row * 3 + col;
            let w = horizontals[wall_index];
            graph.add_edge(id, next_id, w);
        }
    }

    // 3. Ajouter les liens verticaux
    //   - 4 colonnes, 3 "morceaux" par colonne => 12 total
    //   - L'identifiant du nœud (row*3 + col)
    for row in 0..4 {
        for col in 0..3 {
            let id = row * 3 + col;   // nœud courant
            let next_id = id + 3;     // nœud en dessous
            let wall_index = row * 3 + col;
            let w = verticals[wall_index];
            graph.add_edge(id, next_id, w);
        }
    }

    graph
}
