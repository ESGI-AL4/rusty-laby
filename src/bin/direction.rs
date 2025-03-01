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
    pub fn relative_oposite(self, relative_dir: String) -> String {
        let front = "Front".to_string();
        let right = "Right".to_string();
        let back = "Back".to_string();
        let left = "Left".to_string();
        if relative_dir == front {
            return back;
        }

        if relative_dir == right {
            return left;
        }

        if relative_dir == back {
            return front;
        }

        if relative_dir == left {
            return right;
        }
        return front;
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