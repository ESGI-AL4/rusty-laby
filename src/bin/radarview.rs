#[derive(Debug, Clone, Copy)]
pub enum CellNature {
    None,
    Hint,
    Goal,
    Invalid,
}

#[derive(Debug, Clone, Copy)]
pub enum CellEntity {
    None,
    Ally,
    Enemy,
    Monster,
}

#[derive(Debug, Clone, Copy)]
pub struct DecodedCell {
    pub nature: CellNature,
    pub entity: CellEntity,
}

#[derive(Debug, Clone, Copy)]
pub enum Wall {
    Undefined,
    Open,
    Wall,
}

#[derive(Debug)]
pub struct PrettyRadarView {
    pub horizontal_walls: Vec<Wall>,
    pub vertical_walls: Vec<Wall>,
    pub cells: Vec<DecodedCell>,
    pub current_cell: usize,
}

fn encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";
    let mut encoded = String::new();
    let mut i = 0;
    while i < data.len() {
        let mut group = [0u8; 3];
        let remaining = data.len() - i;
        let copy_length = std::cmp::min(remaining, group.len());
        group[..copy_length].copy_from_slice(&data[i..i + copy_length]);

        encoded.push(ALPHABET[(group[0] >> 2) as usize] as char);
        encoded.push(ALPHABET[((group[0] & 0x03) << 4 | group[1] >> 4) as usize] as char);
        if remaining > 1 {
            encoded.push(ALPHABET[((group[1] & 0x0F) << 2 | group[2] >> 6) as usize] as char);
        }
        if remaining > 2 {
            encoded.push(ALPHABET[(group[2] & 0x3F) as usize] as char);
        }

        i += 3;
    }
    encoded
}

fn decode(encoded: String) -> Result<Vec<u8>, String> {
    const REV_ALPHABET: [i8; 128] = {
        let mut table: [i8; 128] = [-1; 128];
        let alphabet = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";
        let mut i = 0;
        while i < alphabet.len() {
            table[alphabet[i] as usize] = i as i8;
            i += 1;
        }
        table
    };

    let mut decoded = Vec::new();
    let mut i = 0;
    while i < encoded.len() {
        let mut group = [0u8; 4];
        for j in 0..4 {
            if i + j < encoded.len(){
                let char_val = encoded.as_bytes()[i + j] as usize;
                if char_val >= REV_ALPHABET.len() || REV_ALPHABET[char_val] == -1 {
                    return Err("Invalid character in encoded string".to_string());
                }
                group[j] = REV_ALPHABET[char_val] as u8;
            }
        }

        decoded.push(group[0] << 2 | group[1] >> 4);
        if i + 2 < encoded.len() {
            decoded.push(group[1] << 4 | group[2] >> 2);
        }
        if i + 3 < encoded.len() {
            decoded.push(group[2] << 6 | group[3]);
        }

        i += 4;
    }

    Ok(decoded)
}

pub fn decode_radar_view(radar_b64: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let bytes = decode(radar_b64.to_string()).map_err(|e| format!("Decode error: {}", e))?;
    if bytes.len() < 11 {
        return Err(format!("RadarView: {} octets reçus, on s’attend à 11", bytes.len()).into());
    }
    let mut horizontals = bytes[0..3].to_vec();
    let mut verticals = bytes[3..6].to_vec();
    let cells = bytes[6..11].to_vec();
    horizontals.reverse();
    verticals.reverse();
    Ok((horizontals, verticals, cells))
}

pub fn decode_walls(bytes: &[u8]) -> Vec<Wall> {
    let mut walls = Vec::with_capacity(12);
    let mut bits_24 = 0u32;
    for &b in bytes {
        bits_24 = (bits_24 << 8) | (b as u32);
    }
    for i in 0..12 {
        let shift = 24 - 2 * (i + 1);
        let val = (bits_24 >> shift) & 0b11;
        walls.push(match val {
            0 => Wall::Undefined,
            1 => Wall::Open,
            2 => Wall::Wall,
            _ => Wall::Undefined,
        });
    }
    walls
}

pub fn decode_one_cell(n: u8) -> DecodedCell {
    if n == 0xF {
        return DecodedCell {
            nature: CellNature::Invalid,
            entity: CellEntity::None,
        };
    }
    let nature_bits = (n >> 2) & 0b11;
    let entity_bits = n & 0b11;
    DecodedCell {
        nature: match nature_bits {
            0b00 => CellNature::None,
            0b01 => CellNature::Hint,
            0b10 => CellNature::Goal,
            _ => CellNature::Invalid,
        },
        entity: match entity_bits {
            0b00 => CellEntity::None,
            0b01 => CellEntity::Ally,
            0b10 => CellEntity::Enemy,
            0b11 => CellEntity::Monster,
            _ => CellEntity::None,
        },
    }
}

fn decode_cells(bytes: &[u8]) -> Vec<DecodedCell> {
    let mut bits_40 = 0u64;
    for &b in bytes {
        bits_40 = (bits_40 << 8) | (b as u64);

    }
    let mut result = Vec::with_capacity(9);
    for i in 0..9 {
        let shift = 36 - 4 * (i);
        let nib = ((bits_40 >> shift) & 0xF) as u8;
        let cell = decode_one_cell(nib);
        result.push(cell);
    }
    result
}

pub fn interpret_radar_view(h: &[u8], v: &[u8], c: &[u8]) -> PrettyRadarView {
    PrettyRadarView {
        horizontal_walls: decode_walls(h),
        vertical_walls: decode_walls(v),
        cells: decode_cells(c)
    }
}

impl PrettyRadarView {
    /// Retourne les voisins visibles (index des cellules voisines et le type de mur)
    pub fn visible_neighbors(&self) -> Vec<(usize, Wall)> {
        let mut neighbors = Vec::new();
        let current_row = self.current_cell / 3;
        let current_col = self.current_cell % 3;

        // Vérifier les voisins haut, bas, gauche et droite
        if current_row > 0 {
            let neighbor_index = self.current_cell - 3;
            neighbors.push((neighbor_index, self.horizontal_walls[self.current_cell - 3]));
        }
        if current_row < 2 {
            let neighbor_index = self.current_cell + 3;
            neighbors.push((neighbor_index, self.horizontal_walls[self.current_cell]));
        }
        if current_col > 0 {
            let neighbor_index = self.current_cell - 1;
            neighbors.push((neighbor_index, self.vertical_walls[self.current_cell - 1]));
        }
        if current_col < 2 {
            let neighbor_index = self.current_cell + 1;
            neighbors.push((neighbor_index, self.vertical_walls[self.current_cell]));
        }

        neighbors
    }
}

