use crate::bin::walls::Walls;
use crate::bin::cellstate::CellState;

/// Représente une cellule du labyrinthe.
///
/// Une cellule possède une configuration de murs et un état indiquant si elle a été visitée.
#[derive(Debug, Clone)]
pub struct Cell {
    /// Les murs délimitant la cellule.
    pub walls: Walls,
    /// L'état de la cellule.
    pub state: CellState,
}

impl Cell {
    /// Crée une nouvelle instance de `Cell`.
    ///
    /// Initialise les murs avec leurs valeurs par défaut et l'état à `NotVisited`.
    pub fn new() -> Self {
        Self {
            walls: Walls::default(),
            state: CellState::NotVisited,
        }
    }
}
