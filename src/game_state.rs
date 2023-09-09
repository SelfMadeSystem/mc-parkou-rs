use std::collections::{HashMap, HashSet, VecDeque};

use valence::prelude::*;

use crate::{
    generation::{generation::Generation, theme::GenerationTheme},
    line::Line3,
    prediction::prediction_state::PredictionState,
    utils::*,
};

#[derive(Component)]
pub struct GameState {
    pub generations: VecDeque<Generation>,
    pub target_y: i32,
    pub direction: JumpDirection,
    pub theme: GenerationTheme,
    pub score: u32,
    pub combo: u32,
    pub stopped_running: bool,
    pub prev_pos: DVec3,
    pub test_state: PredictionState,
    pub line_entities: HashMap<Line3, Entity>,
    pub lines: HashSet<Line3>,
}
