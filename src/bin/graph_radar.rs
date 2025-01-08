use std::collections::HashMap;
use crate::bin::radarview::{CellNature, DecodedCell, Wall};


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

        for col in 0..3 {

            if col < 2 {
                let id = col;
                let next_id = id + 1;
                if let Some(wall) = self.walls.get(&(id, next_id)) {
                    println!("{:?}", wall);
                    ascii.push_str(match wall {
                        Wall::Undefined => "##",
                        Wall::Open => " ",
                        Wall::Wall => "-",
                    });
                }
            }
            ascii.push('•');
        }
        ascii.push_str("\n");

        for row in 0..3 {

            // Affichage des cellules et murs verticaux
            for col in 0..3 {
                let id = row * 3 + col;
                let node = &self.nodes[&id];

                // Cellule
                ascii.push_str(&format!(
                    "{}",
                    match node.cell.nature {
                        CellNature::None => " ",
                        CellNature::Hint => "H",
                        CellNature::Goal => "G",
                        CellNature::Invalid => " ",
                    }
                ));

                // Mur vertical (|) ou espace
                if col < 2 {
                    let next_id = id + 1;
                    if let Some(wall) = self.walls.get(&(id, next_id)) {
                        ascii.push_str(match wall {
                            Wall::Undefined => "|",
                            Wall::Open => " ",
                            Wall::Wall => "|",
                        });
                    }
                }
            }


            ascii.push_str("\n");

            // Lignes des murs horizontaux sous les cellules, sauf la dernière ligne
            if row < 2 {
                for col in 0..3 {
                    let id = row * 3 + col;
                    ascii.push('•'); // Jonction
                    if let Some(next_id) = self.nodes[&id].neighbors.get("down") {
                        if let Some(wall) = self.walls.get(&(id, *next_id)) {
                            ascii.push_str(match wall {
                                Wall::Undefined => "##",
                                Wall::Open => " ",
                                Wall::Wall => "-",
                            });
                        }
                    }
                }
                ascii.push_str("\n");
            }
        }

        for col in 0..3 {
            ascii.push('•');
            if col < 2 {
                let id = 6 + col;
                let next_id = id + 1;
                if let Some(wall) = self.walls.get(&(id, next_id)) {
                    ascii.push_str(match wall {
                        Wall::Undefined => "##",
                        Wall::Open => " ",
                        Wall::Wall => "-",
                    });
                }
            }
        }
        ascii.push_str("\n");

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
}


pub fn build_graph(horizontals: &[Wall], verticals: &[Wall], cells: &[DecodedCell]) -> Graph {
    let mut graph = Graph::new();

    // Ajouter les cellules comme nœuds
    for (i, cell) in cells.iter().enumerate() {
        graph.add_node(i, *cell);
    }

    // Ajouter les murs horizontaux
    for row in 0..3 {
        for col in 0..2 {
            let id = row * 3 + col;
            let next_id = id + 1;
            let wall_index = row * 3 + col;
            graph.add_wall(id, next_id, horizontals[wall_index]);
        }
    }

    // Ajouter les murs verticaux
    for row in 0..2 {
        for col in 0..3 {
            let id = row * 3 + col;
            let next_id = id + 3;
            let wall_index = row * 3 + col;
            graph.add_wall(id, next_id, verticals[wall_index]);
        }
    }

    graph
}