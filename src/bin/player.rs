use crate::bin::direction::Direction;

/// Représente un joueur dans le labyrinthe.
///
/// Le joueur possède une position (x, y), une direction actuelle, un chemin parcouru (liste de positions)
/// ainsi qu'un chemin de directions (liste de mouvements effectués).
#[derive(Debug)]
pub struct Player {
    /// Position horizontale du joueur.
    pub x: i32,
    /// Position verticale du joueur.
    pub y: i32,
    /// Direction actuelle du joueur.
    pub direction: Direction,
    /// Chemin parcouru : positions successives visitées par le joueur.
    pub path: Vec<(i32, i32)>,
    /// Chemin des directions effectuées (sous forme de chaîne de caractères).
    pub directions_path: Vec<String>,
}

impl Default for Player {
    /// Retourne une instance par défaut de `Player`.
    ///
    /// Le joueur est initialisé à la position (0, 0), orienté vers le Nord, avec un chemin vide.
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
    /// Crée une nouvelle instance de `Player` avec la position et la direction spécifiées.
    ///
    /// Le chemin parcouru est initialisé avec la position de départ.
    pub fn new(x: i32, y: i32, direction: Direction) -> Self {
        Self {
            x,
            y,
            direction,
            path: vec![(x, y)],
            directions_path: Vec::new(),
        }
    }

    /// Effectue une rotation vers la gauche pour le joueur.
    pub fn turn_left(&mut self) {
        self.direction = self.direction.turn_left();
    }

    /// Effectue une rotation vers la droite pour le joueur.
    pub fn turn_right(&mut self) {
        self.direction = self.direction.turn_right();
    }

    /// Effectue une rotation de 180 degrés pour le joueur.
    pub fn turn_back(&mut self) {
        self.direction = self.direction.turn_back();
    }

    /// Retourne une copie de l'instance actuelle de `Player`.
    ///
    /// Cette méthode réalise une copie complète de la position, de la direction,
    /// ainsi que des chemins parcourus et des directions enregistrées.
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
