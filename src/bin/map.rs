use std::collections::HashMap;
use crate::bin::radarview::{decode_radar_view, interpret_radar_view};
use crate::bin::radarview::CellNature::Invalid;
// ^ Assurez-vous que le chemin vers `radarview` est correct

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
            Direction::West  => Direction::South,
            Direction::South => Direction::East,
            Direction::East  => Direction::North,
        }
    }

    /// Tourne à droite (par ex. North -> East)
    pub fn turn_right(self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East  => Direction::South,
            Direction::South => Direction::West,
            Direction::West  => Direction::North,
        }
    }

    /// Convertit une direction relative (Front/Right/Back/Left)
    /// en direction absolue, en fonction de l'orientation actuelle.
    pub fn relative_to_absolute(self, relative_dir: &str) -> Self {
        match self {
            Direction::North => match relative_dir {
                "Front" => Direction::North,
                "Right" => Direction::East,
                "Back"  => Direction::South,
                "Left"  => Direction::West,
                _       => Direction::North,
            },
            Direction::East => match relative_dir {
                "Front" => Direction::East,
                "Right" => Direction::South,
                "Back"  => Direction::West,
                "Left"  => Direction::North,
                _       => Direction::East,
            },
            Direction::South => match relative_dir {
                "Front" => Direction::South,
                "Right" => Direction::West,
                "Back"  => Direction::North,
                "Left"  => Direction::East,
                _       => Direction::South,
            },
            Direction::West => match relative_dir {
                "Front" => Direction::West,
                "Right" => Direction::North,
                "Back"  => Direction::East,
                "Left"  => Direction::South,
                _       => Direction::West,
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

impl Player {
    pub fn new(x: i32, y: i32, direction: Direction) -> Self {
        Self { x, y, direction }
    }

    pub fn turn_left(&mut self) {
        self.direction = self.direction.turn_left();
    }

    pub fn turn_right(&mut self) {
        self.direction = self.direction.turn_right();
    }
}

/// État d'une cellule de la carte (visitée ou pas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellState {
    NotVisited,
    Visited,
}

/// État d'un mur : ouvert, mur, ou indéfini.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WallStatus {
    Open,
    Wall,
    Undefined,
}

/// Ensemble des 4 murs d'une cellule.
#[derive(Debug, Clone)]
pub struct Walls {
    pub north: WallStatus,
    pub east:  WallStatus,
    pub south: WallStatus,
    pub west:  WallStatus,
}

impl Default for Walls {
    fn default() -> Self {
        Self {
            north: WallStatus::Undefined,
            east:  WallStatus::Undefined,
            south: WallStatus::Undefined,
            west:  WallStatus::Undefined,
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

    /// Récupère une cellule en lecture seule.
    pub fn get_cell(&self, x: i32, y: i32) -> Option<&Cell> {
        self.grid.get(&(x, y))
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
    }

    /// Convertit un string ("Open", "Wall", etc.) en WallStatus
    fn to_wall_status(s: &str) -> WallStatus {
        match s {
            "Open" => WallStatus::Open,
            "Wall" => WallStatus::Wall,
            _      => WallStatus::Undefined,
        }
    }

    /// Met à jour la carte en fonction d'un RadarView reçu (radar_str)
    /// et ajuste la position/orientation du joueur (si nécessaire).
    ///
    /// - Décode le radar_str via `decode_radar_view`
    /// - Interprète (horizontal_walls, vertical_walls, cells)
    /// - Met à jour la grille (position actuelle + cellules voisines)
    pub fn update_from_radar(&mut self, radar_str: &str, player: &mut Player) {
        // 1) Décode le RadarView
        match decode_radar_view(radar_str) {
            Ok((hlist, vlist, clist)) => {
                // 2) Interprète la structure (murs horizontaux/verticaux + cells)
                let interpreted = interpret_radar_view(&hlist, &vlist, &clist);

                // Les 4 premiers indices : horizontaux (north/south) et verticaux (east/west)
                // peuvent varier selon l'ordre imposé par votre decode.
                // Ici on suppose que horizontal_walls[0] correspond à "north"
                // et horizontal_walls[2] correspond à "south", etc.
                /*let w = Walls {
                    north: Self::to_wall_status(&interpreted.horizontal_walls[0].to_string()),
                    east:  Self::to_wall_status(&interpreted.vertical_walls[1].to_string()),
                    south: Self::to_wall_status(&interpreted.horizontal_walls[2].to_string()),
                    west:  Self::to_wall_status(&interpreted.vertical_walls[3].to_string()),
                };

                // On met à jour la cellule actuelle du joueur (x,y).
                self.update_cell(player.x, player.y, w, CellState::Visited);*/

                // Mise à jour des 9 "cells" autour du joueur (3x3),
                // dont la cellule centrale est la position actuelle (index 4).
                // Vous pouvez changer la logique selon l'indexation que vous utilisez.
                for (i, cell_decoded) in interpreted.cells.iter().enumerate() {
                    if cell_decoded.nature == Invalid {
                        continue;
                    }

                    // Ex. : mapping simplifié d'un 3x3 :
                    // 0 1 2
                    // 3 4 5
                    // 6 7 8
                    // `4` = centre, la position actuelle du joueur.
                    let dx = (i % 3) as i32 - 1; // -1, 0, +1
                    let dy = (i / 3) as i32 - 1; // -1, 0, +1

                    let nx = player.x + dx;
                    let ny = player.y + dy;

                    // On crée la cellule si elle n'existe pas.
                    self.get_cell_mut_or_create(nx, ny);
                    // Potentiellement, vous pouvez stocker d'autres infos
                    // venant de cell_decoded (ex: `entity`).
                }
            }
            Err(e) => {
                eprintln!("Erreur decode_radar_view: {}", e);
            }
        }
    }
}
