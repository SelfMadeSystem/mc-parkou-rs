#![allow(clippy::type_complexity)]

use std::collections::{HashMap, HashSet, VecDeque};

use alt_block::AltBlockParams;
use game_state::GameState;
use generation::block_collection::*;
use generation::generator::{GenerationType, Generator};
use generation::generators::*;
use generation::theme::GenerationTheme;
use prediction::prediction_state::PredictionState;
use utils::JumpDirection;
use valence::entity::block_display;
use valence::prelude::*;
use valence::protocol::sound::{Sound, SoundCategory};
use valence::spawn::IsFlat;

mod alt_block;
mod block_types;
mod game_state;
mod generation;
mod line;
mod prediction;
mod utils;
mod weighted_vec;

const START_POS: BlockPos = BlockPos::new(0, 100, 0);
const DIFF: i32 = 10;
const MIN_Y: i32 = START_POS.y - DIFF;
const MAX_Y: i32 = START_POS.y + DIFF;
const VIEW_DIST: u8 = 32;

pub fn main() {
    App::new()
        .insert_resource(NetworkSettings {
            connection_mode: ConnectionMode::Offline,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_clients,
                reset_clients.after(init_clients),
                manage_chunks.after(reset_clients).before(manage_blocks),
                manage_blocks,
                update_alt_blocks,
                spawn_lines,
                despawn_disconnected_clients,
                cleanup_clients,
            ),
        )
        .add_systems(EventLoopUpdate, (detect_stop_running,))
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    let layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    commands.spawn(layer);
}

fn init_clients(
    mut clients: Query<
        (
            Entity,
            &mut Client,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut IsFlat,
            &mut GameMode,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
    mut commands: Commands,
) {
    for (
        entity,
        mut client,
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut is_flat,
        mut game_mode,
    ) in clients.iter_mut()
    {
        let layer = layers.single();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);

        visible_chunk_layer.0 = entity;
        is_flat.0 = true;
        *game_mode = GameMode::Creative; // TODO: Change to adventure

        client.send_chat_message("Welcome to epic infinite parkour game!".italic());

        let state = GameState {
            generations: VecDeque::new(),
            direction: JumpDirection::DoesntMatter,
            theme: GenerationTheme::new(
                "name".to_owned(),
                BlockCollectionMap::from([
                    (
                        "jump_block",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::GRASS_BLOCK,
                                BlockState::OAK_LOG,
                                BlockState::BIRCH_LOG,
                                BlockState::OAK_LEAVES,
                                BlockState::BIRCH_LEAVES,
                                BlockState::DIRT,
                                BlockState::MOSS_BLOCK,
                            ],
                            uniform: false,
                        }),
                    ),
                    (
                        "ramp_stone",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::STONE,
                                BlockState::COBBLESTONE,
                                BlockState::MOSSY_COBBLESTONE,
                            ],
                            uniform: false,
                        }),
                    ),
                    (
                        "ramp_stone_slab",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::STONE_SLAB,
                                BlockState::COBBLESTONE_SLAB,
                                BlockState::MOSSY_COBBLESTONE_SLAB,
                            ],
                            uniform: false,
                        }),
                    ),
                    (
                        "ramp_wood",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::OAK_PLANKS,
                                BlockState::SPRUCE_PLANKS,
                                BlockState::BIRCH_PLANKS,
                                BlockState::JUNGLE_PLANKS,
                            ],
                            uniform: false,
                        }),
                    ),
                    (
                        "ramp_wood_slab",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::OAK_SLAB,
                                BlockState::SPRUCE_SLAB,
                                BlockState::BIRCH_SLAB,
                                BlockState::JUNGLE_SLAB,
                            ],
                            uniform: false,
                        }),
                    ),
                    (
                        "indoor_walls",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::BRICKS,],
                            uniform: true,
                        }),
                    ),
                    (
                        "indoor_floor",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::COBBLED_DEEPSLATE,],
                            uniform: true,
                        }),
                    ),
                    (
                        "indoor_platforms",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::STONE,],
                            uniform: true,
                        }),
                    ),
                    (
                        "water",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::WATER,],
                            uniform: true,
                        }),
                    ),
                    (
                        "cave",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::STONE,
                                BlockState::COBBLESTONE,
                                BlockState::MOSSY_COBBLESTONE,
                            ],
                            uniform: false,
                        }),
                    ),
                    (
                        "island_grass",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::GRASS_BLOCK,],
                            uniform: true,
                        }),
                    ),
                    (
                        "island_dirt",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::DIRT, BlockState::COARSE_DIRT,],
                            uniform: false,
                        }),
                    ),
                    (
                        "island_stone",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::STONE,
                                BlockState::COBBLESTONE,
                                BlockState::MOSSY_COBBLESTONE,
                                BlockState::DEEPSLATE,
                                BlockState::COBBLED_DEEPSLATE,
                                BlockState::ANDESITE,
                                BlockState::DIORITE,
                                BlockState::GRANITE,
                            ],
                            uniform: false,
                        }),
                    ),
                    (
                        "concrete",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::WHITE_CONCRETE,
                                BlockState::ORANGE_CONCRETE,
                                BlockState::MAGENTA_CONCRETE,
                                BlockState::LIGHT_BLUE_CONCRETE,
                                BlockState::YELLOW_CONCRETE,
                                BlockState::LIME_CONCRETE,
                                BlockState::PINK_CONCRETE,
                                BlockState::GRAY_CONCRETE,
                                BlockState::LIGHT_GRAY_CONCRETE,
                                BlockState::CYAN_CONCRETE,
                                BlockState::PURPLE_CONCRETE,
                                BlockState::BLUE_CONCRETE,
                                BlockState::BROWN_CONCRETE,
                                BlockState::GREEN_CONCRETE,
                                BlockState::RED_CONCRETE,
                                // BlockState::BLACK_CONCRETE, // black has no contrast and is completely invisible at night
                            ],
                            uniform: true,
                        }),
                    ),
                    (
                        "blink_on",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::BLUE_CONCRETE,],
                            uniform: true,
                        }),
                    ),
                    (
                        "blink_off",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::RED_CONCRETE,],
                            uniform: true,
                        }),
                    ),
                    (
                        "platform",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::STONE,
                                BlockState::COBBLESTONE,
                                BlockState::MOSSY_COBBLESTONE,
                                BlockState::OAK_PLANKS,
                                BlockState::SPRUCE_PLANKS,
                                BlockState::BIRCH_PLANKS,
                                BlockState::JUNGLE_PLANKS,
                            ],
                            uniform: false,
                        }),
                    ),
                    (
                        "bridge_side",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::BRICKS],
                            uniform: false,
                        }),
                    ),
                    (
                        "bridge_wall",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::BRICK_WALL],
                            uniform: false,
                        }),
                    ),
                    (
                        "bridge_floor",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::OAK_PLANKS],
                            uniform: false,
                        }),
                    ),
                    (
                        "bridge_pillar",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::STONE_BRICKS],
                            uniform: false,
                        }),
                    ),
                    (
                        "bridge_pillar_stair",
                        BlockCollection(BlockChoice {
                            blocks: weighted_vec![BlockState::STONE_BRICK_STAIRS],
                            uniform: false,
                        }),
                    ),
                ]),
                weighted_vec![
                    (GenerationType::Single("jump_block".to_string()), 30.0),
                    (GenerationType::Ramp("ramp_stone".to_string()), 10.0),
                    (GenerationType::Ramp("ramp_wood".to_string()), 10.0),
                    (
                        GenerationType::Indoor {
                            walls: "indoor_walls".to_string(),
                            floor: Some("indoor_floor".to_string()),
                            platforms: "indoor_platforms".to_string(),
                        },
                        2.0
                    ),
                    (
                        GenerationType::Indoor {
                            walls: "indoor_walls".to_string(),
                            floor: Some("water".to_string()),
                            platforms: "indoor_platforms".to_string(),
                        },
                        2.0
                    ),
                    (
                        GenerationType::Indoor {
                            walls: "indoor_walls".to_string(),
                            floor: None,
                            platforms: "indoor_platforms".to_string(),
                        },
                        1.0
                    ),
                    (GenerationType::Cave("cave".to_string()), 5.0),
                    (GenerationType::Snake("concrete".to_string()), 10.0),
                    (
                        GenerationType::BlinkBlocks {
                            on: "blink_on".to_string(),
                            off: "blink_off".to_string(),
                            delay: 20,
                            overlap: 5,
                        },
                        10.0
                    ),
                    (
                        GenerationType::SingleCustom(SingleCustomPreset {
                            blocks: HashMap::from([
                                (BlockPos::new(0, 0, 0), ("platform".to_string(), vec![])),
                                (BlockPos::new(0, 0, 1), ("platform".to_string(), vec![])),
                                (BlockPos::new(0, 0, 2), ("platform".to_string(), vec![])),
                                (BlockPos::new(0, 1, 2), ("platform".to_string(), vec![])),
                                (BlockPos::new(0, 2, 2), ("platform".to_string(), vec![])),
                                (BlockPos::new(0, 0, 3), ("platform".to_string(), vec![])),
                                (BlockPos::new(0, 0, 4), ("platform".to_string(), vec![])),
                            ]),
                            start_pos: BlockPos::new(0, 0, 0),
                            end_pos: BlockPos::new(0, 0, 4)
                        }),
                        10.0
                    ),
                    (
                        GenerationType::MultiCustom(MultiCustomPreset {
                            presets: HashMap::from([
                                // TODO: Finish bridge. Also, put everything here in a separate file.
                                (
                                    "start".to_string(),
                                    SingularMultiCustomPreset {
                                        preset: SingleCustomPreset {
                                            blocks: HashMap::from([
                                                (
                                                    BlockPos::new(0, 0, 0),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 0),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 0),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(0, 0, 1),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 1),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 1),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(0, 0, 2),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 2),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 2),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(0, 0, 3),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 3),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 3),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                            ]),
                                            start_pos: BlockPos::new(0, 0, 0),
                                            end_pos: BlockPos::new(0, 0, 3)
                                        },
                                        nexts: vec!["middle".to_string()],
                                        fixed_offset: Some(BlockPos::new(0, 0, 3)),
                                    }
                                ),
                                (
                                    "middle".to_string(),
                                    SingularMultiCustomPreset {
                                        preset: SingleCustomPreset {
                                            blocks: HashMap::from([
                                                (
                                                    BlockPos::new(0, 0, 0),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 0),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 0),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(0, 0, 1),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 1),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 1),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(0, 0, 2),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 2),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 2),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(0, 0, 3),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 3),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 3),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-2, 1, 0),
                                                    (
                                                        "bridge_wall".to_string(),
                                                        vec![
                                                            (PropName::North, PropValue::Low),
                                                            (PropName::South, PropValue::Low),
                                                            (PropName::Up, PropValue::False)
                                                        ],
                                                    ),
                                                ),
                                                (
                                                    BlockPos::new(-2, 1, 1),
                                                    (
                                                        "bridge_wall".to_string(),
                                                        vec![
                                                            (PropName::North, PropValue::Low),
                                                            (PropName::South, PropValue::Low),
                                                            (PropName::Up, PropValue::False)
                                                        ],
                                                    ),
                                                ),
                                                (
                                                    BlockPos::new(-2, 1, 2),
                                                    (
                                                        "bridge_wall".to_string(),
                                                        vec![
                                                            (PropName::North, PropValue::Low),
                                                            (PropName::South, PropValue::Low),
                                                            (PropName::Up, PropValue::False)
                                                        ],
                                                    ),
                                                ),
                                                (
                                                    BlockPos::new(-2, 1, 3),
                                                    (
                                                        "bridge_wall".to_string(),
                                                        vec![
                                                            (PropName::North, PropValue::Low),
                                                            (PropName::South, PropValue::Low),
                                                            (PropName::Up, PropValue::False)
                                                        ],
                                                    ),
                                                ),
                                                (
                                                    BlockPos::new(2, 1, 0),
                                                    (
                                                        "bridge_wall".to_string(),
                                                        vec![
                                                            (PropName::North, PropValue::Low),
                                                            (PropName::South, PropValue::Low),
                                                            (PropName::Up, PropValue::False)
                                                        ],
                                                    ),
                                                ),
                                                (
                                                    BlockPos::new(2, 1, 1),
                                                    (
                                                        "bridge_wall".to_string(),
                                                        vec![
                                                            (PropName::North, PropValue::Low),
                                                            (PropName::South, PropValue::Low),
                                                            (PropName::Up, PropValue::False)
                                                        ],
                                                    ),
                                                ),
                                                (
                                                    BlockPos::new(2, 1, 2),
                                                    (
                                                        "bridge_wall".to_string(),
                                                        vec![
                                                            (PropName::North, PropValue::Low),
                                                            (PropName::South, PropValue::Low),
                                                            (PropName::Up, PropValue::False)
                                                        ],
                                                    ),
                                                ),
                                                (
                                                    BlockPos::new(2, 1, 3),
                                                    (
                                                        "bridge_wall".to_string(),
                                                        vec![
                                                            (PropName::North, PropValue::Low),
                                                            (PropName::South, PropValue::Low),
                                                            (PropName::Up, PropValue::False)
                                                        ],
                                                    ),
                                                ),
                                            ]),
                                            start_pos: BlockPos::new(0, 0, 0),
                                            end_pos: BlockPos::new(0, 0, 3)
                                        },
                                        nexts: vec!["middle".to_string()],
                                        fixed_offset: Some(BlockPos::new(0, 0, 3)),
                                    },
                                ),
                                (
                                    "end".to_string(),
                                    SingularMultiCustomPreset {
                                        preset: SingleCustomPreset {
                                            blocks: HashMap::from([
                                                (
                                                    BlockPos::new(0, 0, 0),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 0),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 0),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(0, 0, 1),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 1),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 1),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(0, 0, 2),
                                                    ("bridge_pillar".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 2),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 2),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(0, 0, 3),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(-1, 0, 3),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                                (
                                                    BlockPos::new(1, 0, 3),
                                                    ("bridge_floor".to_string(), vec![]),
                                                ),
                                            ]),
                                            start_pos: BlockPos::new(0, 0, 0),
                                            end_pos: BlockPos::new(0, 0, 3)
                                        },
                                        nexts: vec!["middle".to_string()],
                                        fixed_offset: Some(BlockPos::new(0, 0, 3)),
                                    },
                                )
                            ]),
                            start: weighted_vec!["start".to_string()],
                            end: weighted_vec!["end".to_string()],
                            min_length: 2,
                            max_length: 4,
                        }),
                        10.0
                    ),
                    (
                        GenerationType::Island {
                            grass: "island_grass".to_string(),
                            dirt: "island_dirt".to_string(),
                            stone: "island_stone".to_string(),
                            water: "water".to_string(),
                            min_radius: 4,
                            max_radius: 12,
                            min_point_power: 1.0,
                            max_point_power: 1.75,
                        },
                        10.0
                    ),
                    (
                        GenerationType::ComplexCustom(ComplexGenerator::new(vec![
                            // Only "indoor_walls"
                            //
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Bottom,
                                    ..Default::default()
                                }),
                                connection_bottom: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    ..Default::default()
                                }),
                                connection_left: None,
                                connection_right: None,
                            },
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Left,
                                    ..Default::default()
                                }),
                                connection_bottom: None,
                                connection_left: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    ..Default::default()
                                }),
                                connection_right: None,
                            },
                            // Only "bridge_floor"
                            //
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Bottom,
                                    ..Default::default()
                                }),
                                connection_bottom: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    ..Default::default()
                                }),
                                connection_left: None,
                                connection_right: None,
                            },
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Left,
                                    ..Default::default()
                                }),
                                connection_bottom: None,
                                connection_left: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    ..Default::default()
                                }),
                                connection_right: None,
                            },
                            // "indoor_walls" to "bridge_floor" and vice versa
                            //
                            ComplexTile {
                                connection_top: None,
                                connection_bottom: None,
                                connection_left: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Right,
                                    ..Default::default()
                                }),
                                connection_right: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Left,
                                    ..Default::default()
                                }),
                            },
                            // "indoor_walls" and "bridge_floor" junction
                            //
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Bottom,
                                    ..Default::default()
                                }),
                                connection_bottom: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    ..Default::default()
                                }),
                                connection_left: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Right,
                                    can_start: false,
                                    ..Default::default()
                                }),
                                connection_right: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Left,
                                    can_start: false,
                                    ..Default::default()
                                }),
                            },
                            // Only "bridge_floor" and "indoor_walls" junction
                            //
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Bottom,
                                    can_start: false,
                                    ..Default::default()
                                }),
                                connection_bottom: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    can_start: false,
                                    ..Default::default()
                                }),
                                connection_left: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Right,
                                    ..Default::default()
                                }),
                                connection_right: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Left,
                                    ..Default::default()
                                }),
                            },
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Bottom,
                                    can_start: false,
                                    ..Default::default()
                                }),
                                connection_bottom: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    can_start: false,
                                    ..Default::default()
                                }),
                                connection_left: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Right,
                                    ..Default::default()
                                }),
                                connection_right: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Left,
                                    ..Default::default()
                                }),
                            },
                        ])),
                        100000000000000.0
                    ),
                    (
                        GenerationType::ComplexCustom(ComplexGenerator::new(vec![
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Bottom,
                                    ..Default::default()
                                }),
                                connection_bottom: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    ..Default::default()
                                }),
                                connection_left: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Right,
                                    ..Default::default()
                                }),
                                connection_right: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Left,
                                    ..Default::default()
                                }),
                            },
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Left,
                                    ..Default::default()
                                }),
                                connection_bottom: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Right,
                                    ..Default::default()
                                }),
                                connection_left: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    ..Default::default()
                                }),
                                connection_right: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Bottom,
                                    ..Default::default()
                                }),
                            },
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Bottom,
                                    ..Default::default()
                                }),
                                connection_bottom: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    ..Default::default()
                                }),
                                connection_left: None,
                                connection_right: None,
                            },
                            ComplexTile {
                                connection_top: Some(Connection {
                                    name: "indoor_walls".to_string(),
                                    next_direction: complex_gen::Direction::Left,
                                    ..Default::default()
                                }),
                                connection_bottom: None,
                                connection_left: Some(Connection {
                                    name: "bridge_floor".to_string(),
                                    next_direction: complex_gen::Direction::Top,
                                    ..Default::default()
                                }),
                                connection_right: None,
                            },
                        ])),
                        10000.0
                    ),
                ],
            ),
            score: 0,
            combo: 0,
            target_y: 0,
            stopped_running: false,
            tick: 0,
            alt_block_entities: HashMap::new(),
            prev_alt_block_states: HashMap::new(),
            prev_pos: DVec3::new(
                START_POS.x as f64 + 0.5,
                START_POS.y as f64 + 1.0,
                START_POS.z as f64 + 0.5,
            ),
            test_state: PredictionState::new(
                DVec3::new(
                    START_POS.x as f64 + 0.5,
                    START_POS.y as f64 + 1.0,
                    START_POS.z as f64 + 0.5,
                ),
                DVec3::ZERO,
                0.0,
            ),
            line_entities: HashMap::new(),
            lines: HashSet::new(),
        };

        let layer = ChunkLayer::new(ident!("overworld"), &dimensions, &biomes, &server);

        commands.entity(entity).insert((state, layer));
    }
}

fn reset_clients(
    mut commands: Commands,
    mut clients: Query<(
        &mut Client,
        &mut Position,
        &mut Look,
        &mut GameState,
        &mut ChunkLayer,
    )>,
) {
    for (mut client, mut pos, mut look, mut state, mut layer) in clients.iter_mut() {
        state.test_state.yaw = look.yaw / 180.0 * std::f32::consts::PI;
        state.test_state.vel = pos.0 - state.prev_pos;
        // if state.test_state.vel.y == 0. {
        //     if state.test_state.vel.x == 0. && state.test_state.vel.z == 0. {
        //         state.test_state.vel.x = -0.215 * state.test_state.yaw.sin() as f64;
        //         state.test_state.vel.z = 0.215 * state.test_state.yaw.cos() as f64;
        //     }
        //     state.test_state.vel.y = 0.42f32 as f64;
        // }
        state.test_state.pos = pos.0;
        state.prev_pos = pos.0;

        // state.test_state.draw_particles(32, &mut client);

        // for entity in state.entities.iter() {
        //     commands.entity(*entity).despawn();
        // }

        // state.entities.clear();

        // let command = commands.spawn(SheepEntityBundle {
        //         position: *pos,
        //         layer: *entity_layer_id,
        //         entity_name_visible: NameVisible(true),
        //         ..Default::default()
        //     }).id();

        // state.entities.push(command);

        let mut lines = Vec::new();

        for gen in state.generations.iter() {
            lines.append(&mut gen.lines.clone());

            // lines.append(
            //     &mut get_lines_for_block(bbb.next_params.next_pos)
            // );

            // lines.append(
            //     &mut get_lines_for_block(bbb.next_params.end_pos)
            // );
            // bbb.next_params
            //     .initial_state
            //     .draw_particles(bbb.next_params.ticks as usize, &mut client);

            // particle_outline_block(bbb.next_params.end_pos, Vec3::new(1., 0., 0.), &mut client);
            // particle_outline_block(bbb.next_params.next_pos, Vec3::new(0., 1., 0.), &mut client);
        }

        state.lines = lines.into_iter().collect();

        let out_of_bounds = (pos.0.y as i32) < START_POS.y - 40;

        if out_of_bounds || state.is_added() {
            if out_of_bounds && !state.is_added() {
                client.send_chat_message(
                    "Your score was ".italic()
                        + state
                            .score
                            .to_string()
                            .color(Color::GOLD)
                            .bold()
                            .not_italic(),
                );
            }

            // Init chunks.
            for pos in ChunkView::new(ChunkPos::from_block_pos(START_POS), VIEW_DIST).iter() {
                layer.insert_chunk(pos, UnloadedChunk::new());
            }

            state.score = 0;
            state.combo = 0;
            {
                let state = &mut *state;
                for block in &state.generations {
                    block.remove(
                        &mut layer,
                        &mut state.alt_block_entities,
                        &mut state.prev_alt_block_states,
                        &mut commands,
                    );
                }
            }

            state.generations.clear();
            let gen = Generator::first_in_generation(START_POS, &state.theme);
            gen.place(&mut layer);
            state.generations.push_back(gen);

            for _ in 0..10 {
                generate_next_block(&mut state, &mut layer);
            }

            pos.set([
                START_POS.x as f64 + 0.5,
                START_POS.y as f64 + 1.0,
                START_POS.z as f64 + 0.5,
            ]);
            look.yaw = 0.0;
            look.pitch = 0.0;
        }
    }
}

fn cleanup_clients(
    mut commands: Commands,
    mut disconnected_clients: RemovedComponents<Client>,
    mut query: Query<&mut GameState>,
) {
    for entity in disconnected_clients.iter() {
        if let Ok(mut state) = query.get_mut(entity) {
            for entity in state.line_entities.values() {
                commands.entity(*entity).insert(Despawned);
            }

            for entity in state.alt_block_entities.values() {
                commands.entity(*entity).insert(Despawned);
            }

            state.generations.clear();
        }
    }
}

fn update_alt_blocks(
    mut commands: Commands,
    mut clients: Query<(
        &mut GameState,
        &EntityLayerId,
        &mut ChunkLayer, /* , &Position, &OldPosition */
    )>,
) {
    for (mut state, entity_layer, mut layer /* , pos, old_pos */) in clients.iter_mut() {
        state.tick += 1;

        let params = AltBlockParams { ticks: state.tick };

        let state = &mut *state;

        for gen in state.generations.iter_mut() {
            gen.update_alt_blocks(
                &params,
                &mut state.alt_block_entities,
                &mut state.prev_alt_block_states,
                &mut commands,
                &mut layer,
                entity_layer,
            );
        }
    }
}

fn detect_stop_running(mut event: EventReader<SprintEvent>, mut clients: Query<&mut GameState>) {
    for mut state in clients.iter_mut() {
        if let Some(event) = event.iter().next() {
            if matches!(event.state, SprintState::Stop) {
                state.stopped_running = true;
            }
        }
    }
}

fn spawn_lines(mut commands: Commands, mut clients: Query<(&mut GameState, &EntityLayerId)>) {
    for (mut state, layer) in clients.iter_mut() {
        let mut to_remove = Vec::new();
        for (line, entity) in state.line_entities.iter() {
            if state.lines.contains(line) {
                continue;
            }
            commands.entity(*entity).insert(Despawned);
            to_remove.push(*line);
        }

        for line in to_remove {
            state.line_entities.remove(&line);
        }

        let mut entities = HashMap::new();

        for line in state.lines.iter() {
            if state.line_entities.contains_key(line) {
                continue;
            }
            let mut bundle = line.to_block_display();
            bundle.block_display_block_state = block_display::BlockState(BlockState::STONE);
            bundle.layer = *layer;
            let cmd = commands.spawn(bundle);

            entities.insert(*line, cmd.id());
        }

        state.line_entities.extend(entities);
    }
}

fn manage_blocks(
    mut commands: Commands,
    mut clients: Query<(&mut Client, &Position, &mut GameState, &mut ChunkLayer)>,
) {
    for (client, pos, mut state, mut layer) in clients.iter_mut() {
        if let Some(index) = state
            .generations
            .iter()
            .position(|block| block.has_reached(*pos))
        {
            if index > 0 {
                let mut score = index as u32;

                if !state.generations[index].ordered {
                    score -= 1;
                }

                for i in 0..index {
                    let s = state.generations[i].get_unreached_child_count();
                    score += s;
                }
                {
                    let state = &mut *state;

                    for _ in 0..index {
                        remove_block(state, &mut *layer, &mut commands);
                        generate_next_block(state, &mut layer);
                    }
                }
                reached_thing(state, score, client, pos);
            } else {
                let s = state.generations[0].has_reached_child(*pos);
                if s > 0 {
                    reached_thing(state, s, client, pos);
                }
            }
        }
    }
}

fn reached_thing(
    mut state: Mut<'_, GameState>,
    score: u32,
    mut client: Mut<'_, Client>,
    pos: &Position,
) {
    if state.stopped_running {
        state.combo = 0;
    } else {
        state.combo += score;
    }

    state.score += score;

    let pitch = 0.9 + ((state.combo as f32) - 1.0) * 0.05;
    client.play_sound(
        Sound::BlockNoteBlockBass,
        SoundCategory::Master,
        pos.0,
        1.0,
        pitch,
    );

    if true
        || state.score < 50 && state.score % 10 == 0
        || state.score == 75
        || state.score >= 100 && state.score % 50 == 0
    {
        client.set_title("");
        client.set_subtitle(state.score.to_string().color(Color::LIGHT_PURPLE).bold());
    }
}

fn manage_chunks(mut clients: Query<(&Position, &OldPosition, &mut ChunkLayer), With<Client>>) {
    for (pos, old_pos, mut layer) in &mut clients {
        let old_view = ChunkView::new(old_pos.chunk_pos(), VIEW_DIST);
        let view = ChunkView::new(pos.to_chunk_pos(), VIEW_DIST);

        if old_view != view {
            for pos in old_view.diff(view) {
                layer.remove_chunk(pos);
            }

            for pos in view.diff(old_view) {
                layer.chunk_entry(pos).or_default();
            }
        }
    }
}

fn remove_block(state: &mut GameState, world: &mut ChunkLayer, commands: &mut Commands) {
    let alt_block_entities = &mut state.alt_block_entities;
    let prev_alt_block_states = &mut state.prev_alt_block_states;

    let removed_block = state.generations.pop_front().unwrap();
    removed_block.remove(world, alt_block_entities, prev_alt_block_states, commands);
}

fn generate_next_block(state: &mut GameState, layer: &mut ChunkLayer) {
    let prev_gen = state.generations.back().unwrap();

    if prev_gen.end_state.get_block_pos().y < MIN_Y {
        state.target_y = START_POS.y;
        state.direction = JumpDirection::Up;
    } else if prev_gen.end_state.get_block_pos().y > MAX_Y {
        state.target_y = START_POS.y;
        state.direction = JumpDirection::Down;
    } else {
        match state.direction {
            JumpDirection::Up => {
                if prev_gen.end_state.get_block_pos().y >= state.target_y {
                    state.direction = JumpDirection::DoesntMatter;
                }
            }
            JumpDirection::Down => {
                if prev_gen.end_state.get_block_pos().y <= state.target_y {
                    state.direction = JumpDirection::DoesntMatter;
                }
            }
            _ => {}
        }
    }

    let next_gen = Generator::next_in_generation(state.direction, &state.theme, prev_gen);

    next_gen.place(layer);
    state.generations.push_back(next_gen);

    // Combo System
    state.stopped_running = false;
}
