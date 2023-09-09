use crate::weighted_vec::WeightedVec;

use super::generator::GenerationType;

/// The `GenerationTheme` struct represents a theme for a parkour generation.
///
/// Properties:
///
/// * `name`: The `name` property is a string slice that represents the name of the
/// theme.
/// * `generation_types`: The `generation_types` property is a `WeightedVec<GenerationType>`,
/// which is a vector of elements of type `GenerationType` with associated weights.
/// Each element in the vector is assigned a weight, which determines the probability
/// of that element being chosen.
#[derive(Clone, Debug)]
pub struct GenerationTheme {
    pub name: &'static str,
    pub generation_types: WeightedVec<GenerationType>,
}

impl GenerationTheme {
    pub fn new(name: &'static str, generation_types: WeightedVec<GenerationType>) -> Self {
        Self {
            name,
            generation_types,
        }
    }

    pub fn get_random_generation_type(&self) -> GenerationType {
        self.generation_types.get_random().unwrap().clone()
    }
}
