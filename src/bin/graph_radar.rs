use std::cmp::PartialEq;
use std::collections::HashMap;
use crate::bin::radarview::{CellEntity, CellNature, DecodedCell, PrettyRadarView, Wall};

#[derive(Debug)]
struct Node {
    id: usize,
    cell: DecodedCell,
    neighbors: HashMap<String, usize>, // "left", "right", "up", "down"
}

#[derive(Debug)]
pub struct Graph {
    nodes: HashMap<usize, Node>,
    walls: HashMap<(usize, usize), Wall>, // Connexion entre deux cellules
}

impl PartialEq for &Wall {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Wall::Undefined, Wall::Undefined) => true,
            (Wall::Open, Wall::Open) => true,
            (Wall::Wall, Wall::Wall) => true,
            _ => false,
        }
    }
}

impl Graph {
    fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            walls: HashMap::new(),
        }
    }

    fn add_node(&mut self, id: usize, cell: DecodedCell) {
        self.nodes.insert(
            id,
            Node {
                id,
                cell,
                neighbors: HashMap::new(),
            },
        );
    }

    fn add_wall(&mut self, from: usize, to: usize, wall: Wall) {
        self.walls.insert((from, to), wall);
        if let Some(node) = self.nodes.get_mut(&from) {
            node.neighbors.insert("right".to_string(), to);
        }
        if let Some(node) = self.nodes.get_mut(&to) {
            node.neighbors.insert("left".to_string(), from);
        }
    }

    pub(crate) fn visualize_ascii(&self) -> String {
        let mut ascii = String::new();

        // Première ligne de murs horizontaux
        for col in 0..3 {
            if col > 0 {
                let id = col - 1;
                let next_id = col;
                if let Some(wall) = self.walls.get(&(id, next_id)) {
                    ascii.push_str(match wall {
                        Wall::Undefined => "##",
                        Wall::Open => " ",
                        Wall::Wall => "━",
                    });
                }
            }
            ascii.push('•');
        }
        ascii.push_str("##\n");

        // Lignes centrales : cellules et murs
        for row in 0..3 {
            // Ligne des cellules et murs verticaux
            ascii.push_str("##");
            for col in 0..3 {
                let id = row * 3 + col;
                let node = &self.nodes[&id];

                // Cellule
                ascii.push_str(match node.cell.nature {
                    CellNature::None => " ",
                    CellNature::Hint => "H",
                    CellNature::Goal => "G",
                    CellNature::Invalid => "#",
                });

                // Mur vertical
                if col < 2 {
                    let next_id = id + 1;
                    if let Some(wall) = self.walls.get(&(id, next_id)) {
                        ascii.push_str(match wall {
                            Wall::Undefined => "•",
                            Wall::Open => " ",
                            Wall::Wall => "|",
                        });
                    }
                }
            }
            ascii.push_str("##\n");

            // Ligne des murs horizontaux sous les cellules
            if row < 2 {
                ascii.push_str("##");
                for col in 0..3 {
                    let id = row * 3 + col;

                    // Jonction
                    ascii.push('•');

                    // Mur horizontal
                    if let Some(next_id) = self.nodes[&id].neighbors.get("down") {
                        if let Some(wall) = self.walls.get(&(id, *next_id)) {
                            ascii.push_str(match wall {
                                Wall::Undefined => "•",
                                Wall::Open => "-",
                                Wall::Wall => "━",
                            });
                        }
                    }
                }
                ascii.push_str("##\n");
            }
        }

        // Dernière ligne de murs horizontaux
        ascii.push_str("##");
        for col in 0..3 {
            if col > 0 {
                let id = (2 * 3) + col - 1; // Dernière ligne
                let next_id = id + 1;
                if let Some(wall) = self.walls.get(&(id, next_id)) {
                    ascii.push_str(match wall {
                        Wall::Undefined => "•",
                        Wall::Open => "-",
                        Wall::Wall => "━",
                    });
                }
            }
            ascii.push('•');
        }
        ascii.push_str("##\n");

        ascii
    }


    pub fn log_graph(&self) {
        println!("=== Graph Nodes ===");
        for (id, node) in &self.nodes {
            println!(
                "Node {}: {:?}, Neighbors: {:?}",
                id,
                node.cell,
                node.neighbors
            );
        }

        println!("\n=== Graph Walls ===");
        for ((from, to), wall) in &self.walls {
            println!(
                "Wall between Node {} and Node {}: {:?}",
                from, to, wall
            );
        }
        println!("===================\n");
    }


    /// Reconstruit un `RadarView` à partir du graphe.
    pub fn reconstruct_radar_view(&self) -> PrettyRadarView {
        let mut horizontal_walls = vec![Wall::Undefined; 12]; // 12 murs horizontaux
        let mut vertical_walls = vec![Wall::Undefined; 12];   // 12 murs verticaux
        let mut cells = vec![DecodedCell { nature: CellNature::None, entity: CellEntity::None }; 9]; // 9 cellules

        // Reconstruire les cellules
        for (id, node) in &self.nodes {
            cells[*id] = node.cell;
        }

        // Reconstruire les murs horizontaux
        for row in 0..4 {
            for col in 0..3 {
                let id = row * 3 + col;
                let next_id = id + 1;
                if let Some(wall) = self.walls.get(&(id, next_id)) {
                    let wall_index = row * 3 + col;
                    horizontal_walls[wall_index] = *wall;
                }
            }
        }

        // Reconstruire les murs verticaux
        for row in 0..4 {
            for col in 0..3 {
                let id = row * 3 + col;
                let next_id = id + 3;
                if let Some(wall) = self.walls.get(&(id, next_id)) {
                    let wall_index = row * 3 + col;
                    vertical_walls[wall_index] = *wall;
                }
            }
        }

        PrettyRadarView {
            horizontal_walls,
            vertical_walls,
            cells,
        }
    }
}


pub fn build_graph(horizontals: &[Wall], verticals: &[Wall], cells: &[DecodedCell]) -> Graph {
    let mut graph = Graph::new();
    println!("=== Building Graph ===");
    println!("Horizontals: {:?}", horizontals);
    println!("Verticals: {:?}", verticals);
    println!("Cells: {:?}", cells);

    // Ajouter les cellules comme nœuds
    for (i, cell) in cells.iter().enumerate() {
        graph.add_node(i, *cell);
    }

    // Ajouter les murs horizontaux
    for row in 0..4 {
        for col in 0..3{
            let id = row * 3 + col;
            let next_id = id + 1;
            let wall_index = row * 3 + col;
            graph.add_wall(id, next_id, horizontals[wall_index]);
        }
    }

    // Ajouter les murs verticaux
    for row in 0..4{
        for col in 0..3{
            let id = row * 3 + col;
            let next_id = id + 3;
            let wall_index = row * 3 + col;
            graph.add_wall(id, next_id, verticals[wall_index]);
        }
    }

    graph
}