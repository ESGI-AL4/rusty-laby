use crate::bin::radarview::CellNature::Invalid;
use crate::bin::radarview::{decode_radar_view, interpret_radar_view, PrettyRadarView, Wall};
use std::collections::HashMap;

/// Représente l'orientation possible d'un joueur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    /// Tourne à gauche (par ex. North -> West)
    pub fn turn_left(self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }

    /// Tourne à droite (par ex. North -> East)
    pub fn turn_right(self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    /// Fait demi tour (par ex. North -> South )
    pub fn turn_back(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    /// Convertit une direction relative (Front/Right/Back/Left)
    /// en direction absolue, en fonction de l'orientation actuelle.
    pub fn relative_to_absolute(self, relative_dir: &str) -> Self {
        match self {
            Direction::North => match relative_dir {
                "Front" => Direction::North,
                "Right" => Direction::East,
                "Back" => Direction::South,
                "Left" => Direction::West,
                _ => Direction::North,
            },
            Direction::East => match relative_dir {
                "Front" => Direction::East,
                "Right" => Direction::South,
                "Back" => Direction::West,
                "Left" => Direction::North,
                _ => Direction::East,
            },
            Direction::South => match relative_dir {
                "Front" => Direction::South,
                "Right" => Direction::West,
                "Back" => Direction::North,
                "Left" => Direction::East,
                _ => Direction::South,
            },
            Direction::West => match relative_dir {
                "Front" => Direction::West,
                "Right" => Direction::North,
                "Back" => Direction::East,
                "Left" => Direction::South,
                _ => Direction::West,
            },
        }
    }
}

/// Représente l'état du joueur (sa position et son orientation).
#[derive(Debug)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            direction: Direction::North,
        }
    }
}

impl Player {
    pub fn new(x: i32, y: i32, direction: Direction) -> Self {
        Self {
            x,
            y,
            direction,
        }
    }

    pub fn turn_left(&mut self) {
        self.direction = self.direction.turn_left();
    }

    pub fn turn_right(&mut self) {
        self.direction = self.direction.turn_right();
    }

    pub fn turn_back(&mut self) {
        self.direction = self.direction.turn_back();
    }
}

/// État d'une cellule de la carte (visitée ou pas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellState {
    NotVisited,
    Visited,
}

/// Ensemble des 4 murs d'une cellule.
#[derive(Debug, Clone)]
pub struct Walls {
    pub north: Wall,
    pub east: Wall,
    pub south: Wall,
    pub west: Wall,
}

impl Default for Walls {
    fn default() -> Self {
        Self {
            north: Wall::Undefined,
            east: Wall::Undefined,
            south: Wall::Undefined,
            west: Wall::Undefined,
        }
    }
}

/// Structure d'une cellule : murs + état de visite.
#[derive(Debug, Clone)]
pub struct Cell {
    pub walls: Walls,
    pub state: CellState,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            walls: Walls::default(),
            state: CellState::NotVisited,
        }
    }
}

/// Carte du labyrinthe, stockée dans une HashMap dynamique.
/// Les clés sont les coordonnées (x,y) de chaque cellule.
#[derive(Debug)]
pub struct MazeMap {
    grid: HashMap<(i32, i32), Cell>,
}

impl MazeMap {
    /// Crée une carte vide.
    pub fn new() -> Self {
        Self {
            grid: HashMap::new(),
        }
    }
    fn radarview_offset_to_map_north(
        x: i32,
        y: i32,
        index: usize,
        current_direction: Direction,
    ) -> (i32, i32) {
        // Offsets si le joueur fait face au Nord
        // (index 0 = (x-1, y-1), index 1 = (x, y-1), etc.)
        let offsets_north = [
            (-1, 1),
            (0, 1),
            (1, 1),
            (-1, 0),
            (0, 0),
            (1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];

        // Pour East, on tourne "l'ensemble" 90°
        // (front = x+1 => index 1 => (1,0), etc.)
        let offsets_east = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 0),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        let offsets_south = [
            (1, -1),
            (0, -1),
            (-1, -1),
            (1, 0),
            (0, 0),
            (-1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
        ];

        let offsets_west = [
            (1, 1),
            (1, 0),
            (1, -1),
            (0, 1),
            (0, 0),
            (0, -1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
        ];

        let (dx, dy) = match current_direction {
            Direction::North => offsets_north[index],
            Direction::East => offsets_east[index],
            Direction::South => offsets_south[index],
            Direction::West => offsets_west[index],
        };

        (x + dx, y + dy)
    }

    /// Récupère une cellule en lecture seule.
    pub fn get_cell(&self, x: i32, y: i32) -> Option<&Cell> {
        self.grid.get(&(x, y))
    }

    /// Vérifie si une cellule existe.
    pub fn cell_exists(&self, x: i32, y: i32) -> bool {
        self.grid.contains_key(&(x, y))
    }

    /// Récupère (ou crée) une cellule en écriture.
    fn get_cell_mut_or_create(&mut self, x: i32, y: i32) -> &mut Cell {
        self.grid.entry((x, y)).or_insert_with(Cell::new)
    }

    /// Met à jour une cellule donnée.
    /// Par ex., passer l'état en Visited, mettre à jour certains murs, etc.
    pub fn update_cell(&mut self, x: i32, y: i32, walls: Walls, state: CellState) {
        let cell = self.get_cell_mut_or_create(x, y);
        cell.walls = walls;
        cell.state = state;
        println!("Cell x,y : {},{}", x, y);
        println!("update_cell: {:?}", cell);
        println!("grid: {:?}", self.grid);
    }


    /// Met à jour la carte en fonction d'un RadarView reçu (radar_str)
    /// et ajuste la position/orientation du joueur (si nécessaire).
    ///
    /// - Interprète (horizontal_walls, vertical_walls, cells)
    /// - Met à jour la grille (position actuelle + cellules voisines)
    pub fn update_from_radar(&mut self, radarview: &PrettyRadarView, player: &mut Player) {
        // 2) Interprète la structure (murs horizontaux/verticaux + cells)
        let interpreted = radarview;

        print!("map.rs: ");
        println!("{:?}", interpreted);
        println!("{:?}", player);

        let w = Walls {
            north: interpreted.horizontal_walls[4],
            east:  interpreted.vertical_walls[6],
            south: interpreted.horizontal_walls[7],
            west:  interpreted.vertical_walls[5],
        };

        println!("{:?}", w);


        // On met à jour la cellule actuelle du joueur (x,y).
        self.update_cell(player.x, player.y, w, CellState::Visited);

        // Mise à jour des 9 "cells" autour du joueur (3x3),
        // dont la cellule centrale est la position actuelle (index 4).
        // Vous pouvez changer la logique selon l'indexation que vous utilisez.
        for (i, cell_decoded) in interpreted.cells.iter().enumerate() {
            println!("i: {}", i);
            if i == 4 {
                continue; // On ignore la cellule centrale (le joueur est déjà traité)
            }
            if cell_decoded.nature == Invalid {
                continue;
            }

            // Ex. : mapping simplifié d'un 3x3 :
            // 0 1 2
            // 3 4 5
            // 6 7 8
            // `4` = centre, la position actuelle du joueur.
            // On calcule les offsets en fonction de la direction actuelle du joueur.
            let (dx, dy) = Self::radarview_offset_to_map_north(player.x, player.y, i, player.direction);
            println!("dx, dy: {},{}", dx, dy);

            // On crée la cellule si elle n'existe pas.
            self.get_cell_mut_or_create(dx, dy);

            let mut y = 0;
            if i == 0 || i == 1 || i == 2 {
                y = 0;
            } else if i == 3 || i == 4 || i == 5 {
                y = 1;
            } else if i == 6 || i == 7 || i == 8 {
                y = 2;
            }

            // On recupère les murs de la cellule
            let walls = Walls {
                north: interpreted.horizontal_walls[i],
                east:  interpreted.vertical_walls[i + y + 1],
                south: interpreted.horizontal_walls[i + 3],
                west:  interpreted.vertical_walls[i + y],
            };
            println!("Interpreted walls: {:?}", interpreted);
            println!("walls: {:?}", walls);


            self.update_cell(dx, dy, walls, CellState::NotVisited);

            println!("new cell: {:?}", self.get_cell(dx, dy));

        }
        println!("grid: {:?}", self.grid);
    }
}
