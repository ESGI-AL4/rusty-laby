use base64::{
    alphabet::Alphabet,
    engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig},
    engine::DecodePaddingMode,
    DecodeError, Engine as _,
};

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
}

static CUSTOM_ALPHABET_STR: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";

pub fn decode_custom_b64(s: &str) -> Result<Vec<u8>, DecodeError> {
    let alphabet = Alphabet::new(CUSTOM_ALPHABET_STR)
        .map_err(|_| DecodeError::InvalidByte(0, b'?'))?;
    let config = GeneralPurposeConfig::new()
        .with_decode_padding_mode(DecodePaddingMode::Indifferent)
        .with_decode_allow_trailing_bits(true);
    let engine = GeneralPurpose::new(&alphabet, config);
    engine.decode(s)
}

pub fn decode_radar_view(radar_b64: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let bytes = decode_custom_b64(radar_b64)?;
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
        cells: decode_cells(c),
    }
}
