use crate::bin::radarview::{PrettyRadarView, Wall, DecodedCell, CellNature, CellEntity};

pub fn get_labyrinth_mock() -> Vec<PrettyRadarView> {
    let radar_views: Vec<PrettyRadarView> = vec![
        PrettyRadarView {
            horizontal_walls: vec![
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Open,
                Wall::Open,
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall
            ],

            vertical_walls: vec![
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Wall,
                Wall::Open,
                Wall::Wall
            ],
            cells: vec![
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
            ]
        },
        PrettyRadarView {
            horizontal_walls: vec![
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Open,
                Wall::Open,
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined
            ],

            vertical_walls: vec![
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Wall,
                Wall::Open,
                Wall::Wall,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined
            ],
            cells: vec![
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
            ]
        },
        PrettyRadarView {
            horizontal_walls: vec![
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Open,
                Wall::Open,
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined
            ],

            vertical_walls: vec![
                Wall::Wall,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Wall,
                Wall::Open,
                Wall::Wall,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined
            ],
            cells: vec![
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
            ]
        },

        PrettyRadarView {

            horizontal_walls: vec![
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Open,
                Wall::Open,
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined
            ],

            vertical_walls: vec![
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Undefined,
                Wall::Wall,
                Wall::Wall,
                Wall::Wall,
                Wall::Undefined,
                Wall::Wall,
                Wall::Open,
                Wall::Wall,
                Wall::Undefined
            ],
            cells: vec![
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Goal,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::None,
                    entity: CellEntity::None,
                },
                DecodedCell {
                    nature: CellNature::Invalid,
                    entity: CellEntity::None,
                },
            ],
        }
    ];
    radar_views
}