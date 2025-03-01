use crate::bin::walls::Walls;
use crate::bin::cellstate::CellState;

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