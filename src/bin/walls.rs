use crate::bin::radarview::Wall;
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