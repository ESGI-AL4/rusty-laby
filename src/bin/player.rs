use crate::bin::direction::Direction;

#[derive(Debug)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    /// Chemin parcouru: positions successives
    pub path: Vec<(i32, i32)>,
    pub directions_path: Vec<String>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            direction: Direction::North,
            path: Vec::new(),
            directions_path: Vec::new(),
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
            directions_path: Vec::new(),
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

    //implement Clone trait
    pub fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            direction: self.direction,
            path: self.path.clone(),
            directions_path: self.directions_path.clone(),
        }
    }
}