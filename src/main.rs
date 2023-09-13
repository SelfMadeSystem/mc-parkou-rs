#![allow(clippy::type_complexity)]

use std::collections::{HashMap, HashSet, VecDeque};

use game_state::GameState;
use generation::block_collection::*;
use generation::generator::{GenerationType, Generator};
use generation::theme::GenerationTheme;
use prediction::prediction_state::PredictionState;
use utils::JumpDirection;
use valence::entity::block_display;
use valence::prelude::*;
use valence::protocol::sound::{Sound, SoundCategory};
use valence::spawn::IsFlat;

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
                "name",
                weighted_vec![
                    (
                        GenerationType::Single(BlockCollection(BlockChoice {
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
                        })),
                        10.0
                    ),
                    (
                        GenerationType::Ramp(BlockSlabCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockSlab::new(BlockState::STONE, BlockState::STONE_SLAB),
                                BlockSlab::new(
                                    BlockState::COBBLESTONE,
                                    BlockState::COBBLESTONE_SLAB
                                ),
                                BlockSlab::new(
                                    BlockState::MOSSY_COBBLESTONE,
                                    BlockState::MOSSY_COBBLESTONE_SLAB
                                ),
                            ],
                            uniform: false,
                        })),
                        1.0
                    ),
                    (
                        GenerationType::Ramp(BlockSlabCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockSlab::new(BlockState::OAK_PLANKS, BlockState::OAK_SLAB),
                                BlockSlab::new(BlockState::SPRUCE_PLANKS, BlockState::SPRUCE_SLAB),
                                BlockSlab::new(BlockState::BIRCH_PLANKS, BlockState::BIRCH_SLAB),
                                BlockSlab::new(BlockState::JUNGLE_PLANKS, BlockState::JUNGLE_SLAB),
                            ],
                            uniform: false,
                        })),
                        1.0
                    ),
                    (
                        GenerationType::Indoor(IndoorBlockCollection {
                            walls: BlockCollection(BlockChoice {
                                blocks: weighted_vec![BlockState::BRICKS,],
                                uniform: true
                            }),
                            floor: Some(BlockCollection(BlockChoice {
                                blocks: weighted_vec![BlockState::WATER,],
                                uniform: true
                            })),
                            platforms: BlockSlabCollection(BlockChoice {
                                blocks: weighted_vec![BlockSlab::new(
                                    BlockState::STONE,
                                    BlockState::STONE_SLAB
                                ),],
                                uniform: true
                            })
                        }),
                        1.0
                    ),
                    (
                        GenerationType::Indoor(IndoorBlockCollection {
                            walls: BlockCollection(BlockChoice {
                                blocks: weighted_vec![BlockState::BRICKS,],
                                uniform: true
                            }),
                            floor: Some(BlockCollection(BlockChoice {
                                blocks: weighted_vec![BlockState::COBBLED_DEEPSLATE,],
                                uniform: true
                            })),
                            platforms: BlockSlabCollection(BlockChoice {
                                blocks: weighted_vec![BlockSlab::new(
                                    BlockState::STONE,
                                    BlockState::STONE_SLAB
                                ),],
                                uniform: true
                            })
                        }),
                        1.0
                    ),
                    (
                        GenerationType::Indoor(IndoorBlockCollection {
                            walls: BlockCollection(BlockChoice {
                                blocks: weighted_vec![BlockState::BRICKS,],
                                uniform: true
                            }),
                            floor: None,
                            platforms: BlockSlabCollection(BlockChoice {
                                blocks: weighted_vec![BlockSlab::new(
                                    BlockState::STONE,
                                    BlockState::STONE_SLAB
                                ),],
                                uniform: true
                            })
                        }),
                        1.0
                    ),
                    (
                        GenerationType::Cave(BlockCollection(BlockChoice {
                            blocks: weighted_vec![
                                BlockState::STONE,
                                BlockState::COBBLESTONE,
                                BlockState::MOSSY_COBBLESTONE,
                            ],
                            uniform: false,
                        })),
                        1.0
                    )
                ],
            ),
            score: 0,
            combo: 0,
            target_y: 0,
            stopped_running: false,
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

            for block in &state.generations {
                block.remove(&mut layer)
            }
            state.generations.clear();
            let gen = Generator::first_in_generation(START_POS, &state.theme);
            gen.place(&mut layer);
            state.generations.push_back(gen);

            for _ in 0..10 {
                generate_next_block(&mut state, &mut layer, false);
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
    query: Query<&GameState>,
) {
    for entity in disconnected_clients.iter() {
        if let Ok(state) = query.get(entity) {
            for entity in state.line_entities.values() {
                commands.entity(*entity).insert(Despawned);
            }
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

fn manage_blocks(mut clients: Query<(&mut Client, &Position, &mut GameState, &mut ChunkLayer)>) {
    for (client, pos, mut state, mut layer) in clients.iter_mut() {
        if let Some(index) = state
            .generations
            .iter()
            .position(|block| block.has_reached(*pos))
        {
            if index > 0 {
                for _ in 0..index {
                    generate_next_block(&mut state, &mut layer, true)
                }

                reached_thing(state, index as u32, client, pos);
            } else if state.generations[0].has_reached_child(*pos) {
                state.score += 1;
                reached_thing(state, 1, client, pos);
            }
        }
    }
}

fn reached_thing(mut state: Mut<'_, GameState>, index: u32, mut client: Mut<'_, Client>, pos: &Position) {
    if state.stopped_running {
        state.combo = 0;
    } else {
        state.combo += index;
    }

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

fn generate_next_block(state: &mut GameState, layer: &mut ChunkLayer, in_game: bool) {
    if in_game {
        let removed_block = state.generations.pop_front().unwrap();
        removed_block.remove(layer);

        state.score += 1
    }

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
