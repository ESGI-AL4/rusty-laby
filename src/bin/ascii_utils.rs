use crate::bin::radarview::{CellEntity, CellNature, DecodedCell, PrettyRadarView, Wall};

pub fn visualize_radar_ascii(prv: &PrettyRadarView) -> String {
    let mut out = String::new();
    for row in 0..4 {
        let base = row * 3;
        let walls_slice = &prv.horizontal_walls[base..base + 3];
        for &w in walls_slice {
            match w {
                Wall::Undefined => out.push('#'),
                Wall::Open => out.push(' '),
                Wall::Wall => out.push('-'),
            }
            match w {
                Wall::Undefined => out.push('#'),
                Wall::Open => out.push('•'),
                Wall::Wall => out.push('•'),
            }
        }
        out.push_str("\n");
        if row < 3 {
            let start_v = row * 4;
            let v_slice = &prv.vertical_walls[start_v..start_v + 4];
            for &vw in v_slice {
                match vw {
                    Wall::Undefined => out.push('#'),
                    Wall::Open => out.push(' '),
                    Wall::Wall => out.push('|'),
                }
                match vw {
                    Wall::Undefined => out.push('#'),
                    _ => {}
                }
            }
            out.push_str("\n");
        }
    }
    out
}

pub fn visualize_cells_like_prof(cells: &[DecodedCell]) -> String {
    let mut s = String::new();
    s.push_str("Les cellules (par ligne):\n");
    for row in 0..3 {
        let start = row * 3;
        let slice = &cells[start..start + 3];
        let mut line_items = Vec::new();
        for &decoded in slice {
            let cell_str = format_decoded_cell(decoded);
            line_items.push(cell_str);
        }
        s.push_str(&format!("Ligne {} => {}\n", row + 1, line_items.join(", ")));
    }
    s
}

pub fn format_decoded_cell(c: DecodedCell) -> String {
    let nature_str = match c.nature {
        CellNature::None => "Rien",
        CellNature::Hint => "Hint(H)",
        CellNature::Goal => "Goal(G)",
        CellNature::Invalid => "Undefined",
    };

    let entity_str = match c.entity {
        CellEntity::None => "Rien",
        CellEntity::Ally => "Ally",
        CellEntity::Enemy => "Enemy",
        CellEntity::Monster => "Monster",
    };

    if nature_str == "Undefined" && entity_str == "Rien" {
        return "Undefined".to_string();
    }
    if nature_str == "Rien" && entity_str == "Rien" {
        return "Rien".to_string();
    }

    format!("{} + {}", nature_str, entity_str)
}
