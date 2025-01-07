use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
enum Wall {
    Undefined,
    Open,
    Wall,
}

#[derive(Debug, Clone, Copy)]
enum CellNature {
    None,
    Hint,
    Goal,
    Invalid,
}

#[derive(Debug, Clone, Copy)]
enum CellEntity {
    None,
    Ally,
    Enemy,
    Monster,
}

#[derive(Debug, Clone, Copy)]
struct DecodedCell {
    nature: CellNature,
    entity: CellEntity,
}

#[derive(Debug)]
struct Node {
    id: usize,
    cell: DecodedCell,
    neighbors: HashMap<String, usize>, // "left", "right", "up", "down"
}

#[derive(Debug)]
struct Graph {
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

    fn visualize_ascii(&self) -> String {
        let mut ascii = String::new();

        for row in 0..3 {
            // Ligne horizontale
            for col in 0..3 {
                let id = row * 3 + col;
                let node = &self.nodes[&id];
                ascii.push_str(&format!(
                    "{} ",
                    match node.cell.nature {
                        CellNature::None => ".",
                        CellNature::Hint => "H",
                        CellNature::Goal => "G",
                        CellNature::Invalid => "X",
                    }
                ));

                // Murs horizontaux
                if col < 2 {
                    let next_id = id + 1;
                    if let Some(wall) = self.walls.get(&(id, next_id)) {
                        ascii.push_str(match wall {
                            Wall::Undefined => "•",
                            Wall::Open => " ",
                            Wall::Wall => "-",
                        });
                    }
                }
            }
            ascii.push('\n');

            // Ligne verticale
            if row < 2 {
                for col in 0..3 {
                    let id = row * 3 + col;
                    if let Some(next_id) = self.nodes[&id].neighbors.get("down") {
                        if let Some(wall) = self.walls.get(&(id, *next_id)) {
                            ascii.push_str(match wall {
                                Wall::Undefined => "|",
                                Wall::Open => " ",
                                Wall::Wall => "|",
                            });
                        }
                    }
                    ascii.push(' ');
                }
                ascii.push('\n');
            }
        }

        ascii
    }
}

pub fn build_graph(horizontals: &[Wall], verticals: &[Wall], cells: &[DecodedCell]) -> Graph {
    let mut graph = Graph::new();

    for (i, cell) in cells.iter().enumerate() {
        graph.add_node(i, *cell);
    }

    for row in 0..3 {
        for col in 0..2 {
            let id = row * 3 + col;
            let next_id = id + 1;
            let wall_index = row * 3 + col;
            graph.add_wall(id, next_id, horizontals[wall_index]);
        }
    }

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
