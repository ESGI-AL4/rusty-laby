use std::env;
use serde_json::Value;
use std::string::String;

#[derive(Debug, PartialEq, Clone)]
pub enum Wall {
    Undefined,
    Open,
    Wall,
}

#[derive(Debug, PartialEq)]
pub enum Entity {
    None,
    Ally,
    Enemy,
    Monster,
}

#[derive(Debug, PartialEq)]
pub struct Cell {
    item_type: u8,
}

#[derive(Debug, PartialEq)]
pub struct RadarView {
    pub(crate) horizontal_walls: Vec<Wall>,
    pub(crate) vertical_walls: Vec<Wall>,
    pub(crate) cells: Vec<Cell>,
}

#[allow(unused_variables)]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid usage: radarView <encode|decode> <string>");
        return;
    }

    let operation = &args[1];
    let input = String::from(&args[2]);

    let result = match operation.as_str() {
        "decode" => {
            match decode_radar_view(input) {
                Ok(radar_view) => visualize_radar(&radar_view),
                Err(err) => err,
            }
        },
        "encode"=> "Encoding not implemented yet for full visual output".to_string(),
        _ => "Invalid operation".to_string(),
    };

    println!("{}", result);
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

pub fn decode_radar_view(encoded: String) -> Result<RadarView, String> {
    let decoded_bytes = decode(encoded)?;

    if decoded_bytes.len() != 11 {
        return Err("Invalid decoded data length".to_string());
    }

    let horizontal_walls = decode_walls(&decoded_bytes[0..3]);
    let vertical_walls = decode_walls(&decoded_bytes[3..6]);
    let cells = decode_cells(&decoded_bytes[6..11]);

    Ok(RadarView {
        horizontal_walls,
        vertical_walls,
        cells,
    })
}

fn decode_walls(bytes: &[u8]) -> Vec<Wall> {
    let mut walls = Vec::new();
    for i in 0..12 {
        let byte_index = i / 4;
        let bit_index = (i % 4) * 2;
        let wall_value = (bytes[byte_index] >> bit_index) & 0x03;
        let wall = match wall_value {
            0 => Wall::Undefined,
            1 => Wall::Open,
            2 => Wall::Wall,
            _ => panic!("Invalid wall value"),
        };
        walls.push(wall);
    }
    walls.reverse(); // little endian so we reverse
    walls
}

fn decode_cells(bytes: &[u8]) -> Vec<Cell> {
    let mut cells = Vec::new();
    for i in 0..9 {
        let byte_index = i / 2;
        let nibble_index = (i % 2) * 4;
        let item_type = (bytes[byte_index] >> nibble_index) & 0x0F;
        cells.push(Cell { item_type });
    }
    cells
}

fn visualize_radar(radar_view: &RadarView) -> String {
    let mut output = String::new();
    output.push_str("##");
    for i in 0..3 {
        output.push(match radar_view.horizontal_walls[i] {
            Wall::Undefined => '•',
            Wall::Open => '-',
            Wall::Wall => '━',
        });
        output.push('•');
    }
    output.push_str("##\n");

    for row in 0..3 {
        output.push_str("##");
        for col in 0..4 {
            output.push(match radar_view.vertical_walls[row * 4 + col] {
                Wall::Undefined => '|',
                Wall::Open => ' ',
                Wall::Wall => '|',
            });
        }
        output.push_str("##\n");

        output.push_str("•");
        for col in 0..3 {
            output.push(match radar_view.horizontal_walls[row * 3 + 3 + col] {
                Wall::Undefined => '•',
                Wall::Open => '-',
                Wall::Wall => '━',
            });
            output.push('•');
        }
        output.push_str("\n");
    }

    output.push_str("##");
    for i in 9..12 {
        output.push(match radar_view.vertical_walls[i] {
            Wall::Undefined => '|',
            Wall::Open => ' ',
            Wall::Wall => '|',
        });
    }
    output.push_str("##\n");

    output
}

#[cfg(test)]
mod tests {
    use crate::radar_view::Wall::{Open, Undefined, Wall};
    use std::string::String;
    use super::*;

    #[test]
    fn test_decode_radar_view() {
        let encoded = String::from("ieysGjGO8papd/a");
        let radar_view = decode_radar_view(encoded).unwrap();

        assert_eq!(radar_view.horizontal_walls.len(), 12);
        assert_eq!(radar_view.horizontal_walls, vec![
            Undefined, Open, Undefined,
            Wall, Open, Undefined,
            Open, Wall, Undefined,
            Wall, Undefined, Undefined
        ]);

        assert_eq!(radar_view.vertical_walls.len(), 12);
        assert_eq!(radar_view.vertical_walls, vec![
            Undefined, Wall, Wall, Undefined,
            Wall, Open, Wall, Undefined,
            Wall, Undefined, Undefined, Undefined
        ]);

        assert_eq!(radar_view.cells.len(), 9);
        assert_eq!(radar_view.cells, vec![
            Cell { item_type: 0 }, Cell { item_type: 15 }, Cell { item_type: 0 },
            Cell { item_type: 15 }, Cell { item_type: 15 }, Cell { item_type: 0 },
            Cell { item_type: 15 }, Cell { item_type: 0 }, Cell { item_type: 0 }
        ]);
    }


    #[test]
    fn test_decode_visualize() {
        let encoded = String::from("ieysGjGO8papd/a");
        let radar_view = decode_radar_view(encoded).unwrap();
        let visualization = visualize_radar(&radar_view);

        let expected_visualization = "\
        ##•-• •-•##\n\
        ##|   |##\n\
        •-• •-•\n\
        ##|   |##\n\
        • • •\n\
        ##|   |##\n\
        •-• •-•\n\
        ##|   |##\n\
        ";

        assert_eq!(visualization, expected_visualization);
    }
    #[test]
    fn test_encode_decode() {
        assert_eq!(encode(&[0]), "aa");
        assert_eq!(encode(&[25]), "gq");
        assert_eq!(encode(&[26]), "gG");
        assert_eq!(encode(&[51]), "mW");
        assert_eq!(encode(&[52]), "na");
        assert_eq!(encode(&[61]), "pq");
        assert_eq!(encode(&[62]), "pG");
        assert_eq!(encode(&[63]), "pW");
        assert_eq!(encode(b"Hello, World!"), "sgvSBg8SifDVCMXKiq");
        assert_eq!(encode(&[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159,160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175,176,177,178,179,180,181,182,183,184,185,186,187,188,189,190,191,192,193,194,195,196,197,198,199,200,201,202,203,204,205,206,207,208,209,210,211,212,213,214,215,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230,231,232,233,234,235,236,237,238,239,240,241,242,243,244,245,246,247,248,249,250,251,252,253,254,255]), "aaecaWqfbGCicqOlda0odXareHmufryxgbKAgXWDhH8GisiJjcuMjYGPkISSls4VmdeYmZq1nJC4otO7pd0+p0bbqKneruzhseLks0XntK9quvjtvfvwv1HzwLTCxv5FygfIy2rLzMDOAwPRBg1UB3bXCNn0Dxz3EhL6E3X9FN+aGykdHiwgH4IjIOUmJy6pKjgsK5svLPEyMzQBNj2EN6cHOQoKPAANQkMQQ6YTRQ+WSBkZTlw2T7I5URU8VB6/WmhcW8tfXSFiYCRlZm3oZ9dr0Tpu1DBx2nNA29ZD3T/G4ElJ5oxM5+JP6UVS7E7V8phY8/t19VF4+FR7/p3+/W");

        let test_string = "Hello Radar!";
        let encoded = encode(test_string.as_bytes());
        let decoded = decode(encoded).unwrap();
        assert_eq!(decoded, test_string.as_bytes());

        let test_string = "";
        let encoded = encode(test_string.as_bytes());
        let decoded = decode(encoded).unwrap();
        assert_eq!(decoded, test_string.as_bytes());

        let test_string = "This is a longer test string to really see if everything is working fine.";
        let encoded = encode(test_string.as_bytes());
        let decoded = decode(encoded).unwrap();
        assert_eq!(decoded, test_string.as_bytes());
    }

    #[test]
    fn test_decode_invalid_char() {
        assert!(decode(String::from("abçd")).is_err());
    }
}