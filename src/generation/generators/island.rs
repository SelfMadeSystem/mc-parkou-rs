use std::collections::HashMap;

use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    *,
};
use rand::Rng;
use valence::prelude::*;

use crate::generation::generator::{BlockGenParams, BlockGenerator, GenerateResult};

pub struct IslandGenerator {
    pub grass: String,
    pub dirt: String,
    pub stone: String,
    pub water: String,
    pub min_radius: i32,
    pub max_radius: i32,
    pub min_point_power: f32, // 1f32..1.75f32
    pub max_point_power: f32,
}

impl BlockGenerator for IslandGenerator {
    fn generate(&self, params: &BlockGenParams) -> GenerateResult {
        let mut rng = rand::thread_rng();
        let radius = rng.gen_range(self.min_radius..=self.max_radius);
        let pow = rng.gen_range(self.min_point_power..=self.max_point_power);

        fn get_x_radius(radius: i32, z: i32) -> i32 {
            let size = radius as f32;
            let mut z = z as f32;
            if z == -size {
                z = 0.25 - size;
            } else if z == size {
                z = size - 0.25;
            }
            (size * size - z * z).sqrt().round() as i32
        }

        fn get_dist_to_center(x: i32, z: i32) -> f32 {
            let x = x as f32;
            let z = z as f32;
            (x * x + z * z).sqrt()
        }

        let mut blocks = HashMap::new();

        let mut fbm = Fbm::<SuperSimplex>::new(rng.gen());
        fbm.octaves = 4;
        fbm.frequency = 0.5;
        fbm.persistence = 0.5;
        fbm.lacunarity = 2.;
        let a = radius as f64 / 10.;
        let map: NoiseMap = PlaneMapBuilder::<_, 2>::new(&fbm)
            .set_size((radius * 2 + 1) as usize, (radius * 2 + 1) as usize)
            .set_x_bounds(-a, a)
            .set_y_bounds(-a, a)
            .set_is_seamless(true)
            .build();

        fn get_height(map: &NoiseMap, x: i32, z: i32) -> i32 {
            (map.get_value(z as usize, (x + map.size().1 as i32 / 2) as usize) * 5.0).round() as i32
        }

        let (avg_end_height, (max_end_height, max_end_height_x), (min_start, min_start_x)) = {
            let mut sum = 0;
            let mut max = i32::MIN;
            let mut max_x = 0i32;

            let mut min = i32::MAX;
            let mut min_x = 0i32;

            let z = radius * 2;

            let s = get_x_radius(radius, z - radius);

            for x in -s..=s {
                {
                    let y = get_height(&map, x, z);

                    sum += y;

                    if y > max || (y == max && x.abs() < max_x.abs()) {
                        max = y;
                        max_x = x;
                    }
                }

                {
                    let y = get_height(&map, x, 0);

                    if y < min || (y == min && x.abs() < min_x.abs()) {
                        min = y;
                        min_x = x;
                    }
                }
            }

            (sum / (s * 2 + 1), (max, max_x), (min, min_x))
        };

        let pos = BlockPos {
            x: -min_start_x,
            y: -min_start,
            z: 0,
        };

        let mut min_y = i32::MAX;

        for z in 0..=radius * 2 {
            let s = get_x_radius(radius, z - radius);
            for x in -s..=s {
                let y = get_height(&map, x, z);

                let pos = BlockPos {
                    x: pos.x + x,
                    y: pos.y + y,
                    z: pos.z + z,
                };

                if y < avg_end_height {
                    for y in 1..=(avg_end_height - y) {
                        let pos = BlockPos {
                            x: pos.x,
                            y: pos.y + y,
                            z: pos.z,
                        };
                        blocks.insert(pos, params.block_map.get_block(&self.water));
                    }

                    blocks.insert(pos, params.block_map.get_block(&self.dirt));
                } else {
                    blocks.insert(pos, params.block_map.get_block(&self.grass));
                }

                {
                    let pos = BlockPos {
                        x: pos.x,
                        y: pos.y - 1,
                        z: pos.z,
                    };
                    blocks.insert(pos, params.block_map.get_block(&self.dirt));
                }

                if y < min_y {
                    min_y = y;
                }
            }
        }

        for z in 0..=radius * 2 {
            let s = get_x_radius(radius, z - radius);
            for x in -s..=s {
                let dist = get_dist_to_center(x, z - radius);

                let y = get_height(&map, x, z);

                let mut down_to = min_y - (radius as f32 - dist + 1.).powf(pow).round() as i32;

                down_to += (((y - min_y) as f32) * (dist / radius as f32).powf(2.)) as i32;

                if down_to < y {
                    for y in down_to..y {
                        let pos = BlockPos {
                            x: pos.x + x,
                            y: pos.y + y - 1,
                            z: pos.z + z,
                        };
                        blocks.insert(pos, params.block_map.get_block(&self.stone));
                    }
                }
            }
        }

        let end_pos = BlockPos {
            x: pos.x + max_end_height_x,
            y: pos.y + max_end_height,
            z: pos.z + radius * 2,
        };

        GenerateResult {
            blocks,
            start: BlockPos::new(0, 0, 0),
            end: end_pos,
            alt_blocks: HashMap::new(),
            lines: vec![],
            children: vec![],
        }
    }
}
