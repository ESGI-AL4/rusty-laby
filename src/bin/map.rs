use crate::bin::radarview::CellNature::Invalid;
use crate::bin::radarview::{decode_radar_view, interpret_radar_view, PrettyRadarView, Wall};
use std::collections::HashMap;
use piston_window::{Context, G2d, rectangle, line};
use piston_window::color::BLUE;

/// Quelques couleurs
const COLOR_VISITED:    [f32;4] = [1.0, 0.65, 0.0, 1.0]; // orange
const COLOR_NOTVISITED: [f32;4] = [1.0, 1.0, 1.0, 1.0];  // blanc
const COLOR_PLAYER:     [f32;4] = [0.0, 1.0, 0.0, 1.0];  // vert
const COLOR_UNKNOWN:    [f32;4] = [0.2, 0.2, 0.2, 1.0];  // gris
const COLOR_WALL:       [f32;4] = [0.0, 0.0, 0.0, 1.0];  // noir
const COLOR_PATH: [f32; 4] = [0.0, 0.0, 1.0, 1.0]; // bleu opaque


/// Taille en pixels d'une cellule
pub const CELL_SIZE: f64 = 16.0;
/// Taille en pixels de l'espace/mur
pub const GAP: f64 = 4.0;

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

    //Nouvelle position du joueur après un mouvement
    pub fn new_position(self, x: i32, y: i32, action: &str) -> (i32, i32) {
        match action {
            "Front" => {
                let (dx, dy) = match self {
                    Direction::North => (0, 1),
                    Direction::East => (1, 0),
                    Direction::South => (0, -1),
                    Direction::West => (-1, 0),
                };
                (x + dx, y + dy)
            }
            "Back" => {
                let (dx, dy) = match self {
                    Direction::North => (0, -1),
                    Direction::East => (-1, 0),
                    Direction::South => (0, 1),
                    Direction::West => (1, 0),
                };
                (x + dx, y + dy)
            }
            "Right" => {
                let (dx, dy) = match self {
                    Direction::North => (1, 0),
                    Direction::East => (0, -1),
                    Direction::South => (-1, 0),
                    Direction::West => (0, 1),
                };
                (x + dx, y + dy)
            }
            "Left" => {
                let (dx, dy) = match self {
                    Direction::North => (-1, 0),
                    Direction::East => (0, 1),
                    Direction::South => (1, 0),
                    Direction::West => (0, -1),
                };
                (x + dx, y + dy)
            }
            _ => (x, y),
        }
    }
}

/// Représente l'état du joueur (sa position et son orientation).
#[derive(Debug)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    /// Chemin parcouru: positions successives
    pub path: Vec<(i32, i32)>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            direction: Direction::North,
            path: Vec::new(),
        }
    }
}

impl Player {
    pub fn new(x: i32, y: i32, direction: Direction) -> Self {
        Self {
            x,
            y,
            direction,
            path: vec![(x, y)],
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

fn merge_wall(old: Wall, new: Wall) -> Wall {
    println!("old: {:?}", old);
    println!("new: {:?}", new);
    match (old, new) {
        (Wall::Undefined, Wall::Undefined) => Wall::Undefined,
        (Wall::Undefined, Wall::Wall) => Wall::Wall,
        (Wall::Undefined, Wall::Open) => Wall::Open,
        (Wall::Wall, Wall::Undefined) => Wall::Wall,
        (Wall::Wall, Wall::Wall) => Wall::Wall,
        (Wall::Wall, Wall::Open) => Wall::Wall,
        (Wall::Open, Wall::Undefined) => Wall::Open,
        (Wall::Open, Wall::Wall) => Wall::Open,
        (Wall::Open, Wall::Open) => Wall::Open,
    }
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
    pub fn update_cell(&mut self, x: i32, y: i32, new_walls: Walls, new_state: CellState) {
        let cell = self.get_cell_mut_or_create(x, y);

        // Fusion des murs
        cell.walls.north = merge_wall(cell.walls.north, new_walls.north);
        cell.walls.east  = merge_wall(cell.walls.east,  new_walls.east);
        cell.walls.south = merge_wall(cell.walls.south, new_walls.south);
        cell.walls.west  = merge_wall(cell.walls.west,  new_walls.west);
        println!("update_cell: {:?}", cell);

        // Conserver l’état Visited si déjà visité
        if cell.state == CellState::Visited {
            // On ne touche pas
        } else {
            // Si on n’était pas Visited, on applique le new_state
            cell.state = new_state;
        }

        //println!("Cell x,y : {},{}", x, y);
        //println!("update_cell: {:?}", cell);
        //println!("grid: {:?}", self.grid);
    }

    /// Update the player position and direction after a move
    pub fn update_player(&mut self, player: &mut Player, action: &str) {
        let (x, y) = player.direction.new_position(player.x, player.y, action);

        // 2) Chercher si (x, y) est déjà dans path
        if let Some(pos_index) = player.path.iter().position(|&(px, py)| px == x && py == y) {
            // => On a trouvé (nx, ny) dans la pile, à l'index pos_index

            // On retire tout ce qui est après pos_index
            player.path.truncate(pos_index + 1);
            // i.e. si pos_index = 3, on supprime tout au-delà de l'index 3,
            // ce qui ramène la pile au moment exact où on était à (nx, ny).

        } else {
            // => (nx, ny) n'était pas dans la pile => c'est une nouvelle position
            player.path.push((x, y));
        }

        player.x = x;
        player.y = y;
        player.direction = player.direction.relative_to_absolute(action);
    }

    /// Met à jour la carte en fonction d'un RadarView reçu (radar_str)
    /// et ajuste la position/orientation du joueur (si nécessaire).
    ///
    /// - Interprète (horizontal_walls, vertical_walls, cells)
    /// - Met à jour la grille (position actuelle + cellules voisines)
    pub fn update_from_radar(&mut self, radarview: &PrettyRadarView, player: &mut Player) {
        // 2) Interprète la structure (murs horizontaux/verticaux + cells)
        let interpreted = radarview;

        //print!("map.rs: ");
        //println!("{:?}", interpreted);
        //println!("{:?}", player);

        let w = Walls {
            north: interpreted.horizontal_walls[4],
            east:  interpreted.vertical_walls[6],
            south: interpreted.horizontal_walls[7],
            west:  interpreted.vertical_walls[5],
        };

        //println!("{:?}", w);


        // On met à jour la cellule actuelle du joueur (x,y).
        self.update_cell(player.x, player.y, w, CellState::Visited);

        // Mise à jour des 9 "cells" autour du joueur (3x3),
        // dont la cellule centrale est la position actuelle (index 4).
        // Vous pouvez changer la logique selon l'indexation que vous utilisez.
        for (i, cell_decoded) in interpreted.cells.iter().enumerate() {
            //println!("i: {}", i);
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
            //println!("dx, dy: {},{}", dx, dy);

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
            //println!("Interpreted walls: {:?}", interpreted);
            //println!("walls: {:?}", walls);


            self.update_cell(dx, dy, walls, CellState::NotVisited);

            //println!("new cell: {:?}", self.get_cell(dx, dy));

        }
        //println!("grid: {:?}", self.grid);
    }

    /// Fonction pour afficher la carte dans la console.
    pub fn display_map(&self, player_position: Option<(i32, i32)>, player: &Player) {
        // On récupère la grille en référence immuable pour ne pas la déplacer
        let grid = &self.grid;

        if grid.is_empty() {
            println!("La carte est vide.");
            return;
        }

        // Déterminer les limites de la carte
        let min_x = grid.keys().map(|&(x, _)| x).min().unwrap();
        let max_x = grid.keys().map(|&(x, _)| x).max().unwrap();
        let min_y = grid.keys().map(|&(_, y)| y).min().unwrap();
        let max_y = grid.keys().map(|&(_, y)| y).max().unwrap();

        // Taille de la carte
        let width = (max_x - min_x + 1) as usize;
        let height = (max_y - min_y + 1) as usize;

        // Initialiser une grille de caractères ASCII
        // On prévoit (width * 3 + 1) colonnes, (height * 2 + 1) lignes
        let mut ascii_grid: Vec<Vec<char>> = vec![vec![' '; (width * 3 + 1)]; (height * 2 + 1)];

        // Parcourt toutes les positions du min au max
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let cell_opt = grid.get(&(x, y));

                // On calcule les indices dans la grille ASCII.
                // Le "grid_x" se base sur (x - min_x), et "grid_y" se base sur (max_y - y)
                // pour que la case (min_x, max_y) soit en haut à gauche.
                let grid_x = ((x - min_x) * 3) as usize;
                let grid_y = ((max_y - y) * 2) as usize;

                if let Some(cell) = cell_opt {
                    // MUR NORD
                    match cell.walls.north {
                        Wall::Wall => {
                            ascii_grid[grid_y][grid_x + 1] = '-';
                            ascii_grid[grid_y][grid_x + 2] = '-';
                        }
                        Wall::Open | Wall::Undefined => {
                            ascii_grid[grid_y][grid_x + 1] = ' ';
                            ascii_grid[grid_y][grid_x + 2] = ' ';
                        }
                    }

                    // MUR OUEST
                    match cell.walls.west {
                        Wall::Wall => {
                            ascii_grid[grid_y + 1][grid_x] = '|';
                        }
                        Wall::Open | Wall::Undefined => {
                            ascii_grid[grid_y + 1][grid_x] = ' ';
                        }
                    }

                    // MUR EST
                    match cell.walls.east {
                        Wall::Wall => {
                            ascii_grid[grid_y + 1][grid_x + 3] = '|';
                        }
                        Wall::Open | Wall::Undefined => {
                            ascii_grid[grid_y + 1][grid_x + 3] = ' ';
                        }
                    }

                    // MUR SUD
                    match cell.walls.south {
                        Wall::Wall => {
                            ascii_grid[grid_y + 2][grid_x + 1] = '-';
                            ascii_grid[grid_y + 2][grid_x + 2] = '-';
                        }
                        Wall::Open | Wall::Undefined => {
                            ascii_grid[grid_y + 2][grid_x + 1] = ' ';
                            ascii_grid[grid_y + 2][grid_x + 2] = ' ';
                        }
                    }

                    // Caractère pour l'état de la cellule
                    let cell_char = match cell.state {
                        CellState::Visited => '🟧',
                        CellState::NotVisited => '⬜',
                    };

                    // Si on a un joueur et qu'il est à cette position
                    let display_char = if let Some(pos) = player_position {
                        if pos == (x, y) {
                            '🟩'
                        } else {
                            cell_char
                        }
                    } else {
                        cell_char
                    };

                    ascii_grid[grid_y + 1][grid_x + 1] = display_char;
                } else {
                    // Cellule inexistante => on laisse des espaces
                    // (on pourrait mettre un '?' ou autre pour visualiser les trous)
                    ascii_grid[grid_y][grid_x + 1] = ' ';
                    ascii_grid[grid_y + 1][grid_x] = ' ';
                    ascii_grid[grid_y + 1][grid_x + 1] = ' ';
                    ascii_grid[grid_y + 1][grid_x + 3] = ' ';
                    ascii_grid[grid_y + 2][grid_x + 1] = ' ';
                    ascii_grid[grid_y + 2][grid_x + 2] = ' ';
                }
            }
        }

        // On parcourt player.path
        for &(px, py) in &player.path {
            // Vérifie si la case (px, py) existe dans la map
            // ou si elle est dans les limites.
            if px < min_x || px > max_x || py < min_y || py > max_y {
                continue; // en dehors du rectangle actuel
            }
            let grid_x = ((px - min_x) * 3) as usize;
            let grid_y = ((max_y - py) * 2) as usize;
            ascii_grid[grid_y + 1][grid_x + 1] = '🟦';
        }

        // Dessiner la bordure droite (mur de l'est) pour la dernière colonne
        for row in 0..=(height * 2) {
            let col = (width * 3) as usize;
            ascii_grid[row][col] = '|';
        }

        // Dessiner la bordure supérieure
        for col in 0..=width {
            let xx = (col * 3) as usize;
            let yy = 0;
            ascii_grid[yy][xx] = '+';
        }
        // Dessiner la bordure inférieure
        for col in 0..=width {
            let xx = (col * 3) as usize;
            let yy = (height * 2) as usize;
            ascii_grid[yy][xx] = '+';
        }

        // Dessiner les intersections internes
        for row in 0..=height * 2 {
            for col in 0..=width * 3 {
                if row % 2 == 0 && col % 3 == 0 {
                    ascii_grid[row][col] = '+';
                }
            }
        }

        // Imprimer la grille ASCII ligne par ligne
        for row in ascii_grid {
            let line: String = row.into_iter().collect();
            println!("{}", line);
        }
    }

    pub fn draw_piston(&self, c: Context, g: &mut G2d,
                       player_x: i32, player_y: i32, player: &Player)
    {
        // si vide => rien
        if self.grid.is_empty() { return; }

        let min_x = self.grid.keys().map(|&(x,_ )| x).min().unwrap();
        let max_x = self.grid.keys().map(|&(x,_ )| x).max().unwrap();
        let min_y = self.grid.keys().map(|&(_,y )| y).min().unwrap();
        let max_y = self.grid.keys().map(|&(_,y )| y).max().unwrap();

        // On parcourt toutes les cellules
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // px, py = position en pixels (haut-gauche)
                let px = (x - min_x) as f64 * (CELL_SIZE + GAP);
                let py = (max_y - y) as f64 * (CELL_SIZE + GAP);

                if let Some(cell) = self.grid.get(&(x, y)) {
                    // Couleur de base
                    let mut color = match cell.state {
                        CellState::Visited => COLOR_VISITED,
                        CellState::NotVisited => COLOR_NOTVISITED,
                    };
                    if (x, y) == (player_x, player_y) {
                        color = COLOR_PLAYER;
                    }

                    // 1) Dessin de la zone principale (CELL_SIZE x CELL_SIZE)
                    rectangle(color, [px, py, CELL_SIZE, CELL_SIZE], c.transform, g);

                    // 2) Mur ou espace "east"
                    //    => rectangle de GAP en largeur, la même hauteur que la cellule
                    let east_x = px + CELL_SIZE;
                    let east_y = py;
                    let east_color = if matches!(cell.walls.east, Wall::Wall) {
                        COLOR_WALL
                    } else {
                        color
                    };
                    rectangle(east_color, [east_x, east_y, GAP, CELL_SIZE], c.transform, g);

                    // 3) Mur ou espace "south"
                    // => rectangle horizontal, occupant la totalité de la largeur (CELL_SIZE + GAP)
                    let south_x = px;
                    let south_y = py + CELL_SIZE;
                    let south_width = CELL_SIZE + GAP;
                    let south_color = if matches!(cell.walls.south, Wall::Wall) {
                        COLOR_WALL
                    } else {
                        color
                    };
                    rectangle(south_color, [south_x, south_y, south_width, GAP],
                              c.transform, g);

                } else {
                    // Cellule inconnue => tout en gris
                    rectangle(COLOR_UNKNOWN,
                              [px, py, CELL_SIZE, CELL_SIZE],
                              c.transform, g);

                    // On peint l'espace east en "fond"
                    rectangle(COLOR_UNKNOWN,
                              [px + CELL_SIZE, py, GAP, CELL_SIZE],
                              c.transform, g);
                    // l'espace south
                    rectangle(COLOR_UNKNOWN,
                              [px, py+CELL_SIZE, CELL_SIZE+GAP, GAP],
                              c.transform, g);
                }
            }
        }

        // Ensuite on dessine éventuellement le chemin
        // => exemple : tracer un segment ou remplir en bleu
        // (vous pouvez ajuster car la position d'une cellule n'est plus " x * CELL_SIZE"
        //  mais " x * (CELL_SIZE + GAP )".)
        for w in player.path.windows(2) {
            let (x1, y1) = w[0];
            let (x2, y2) = w[1];

            let sx1 = (x1 - min_x) as f64 * (CELL_SIZE + GAP) + CELL_SIZE/2.0;
            let sy1 = (max_y - y1) as f64 * (CELL_SIZE + GAP) + CELL_SIZE/2.0;
            let sx2 = (x2 - min_x) as f64 * (CELL_SIZE + GAP) + CELL_SIZE/2.0;
            let sy2 = (max_y - y2) as f64 * (CELL_SIZE + GAP) + CELL_SIZE/2.0;

            line(BLUE, 3.0, [sx1, sy1, sx2, sy2], c.transform, g);
        }
    }


}
