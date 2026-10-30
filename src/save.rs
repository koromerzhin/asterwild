use serde::{Deserialize, Serialize};

use crate::resources::{Inventory, Score};
use crate::world::ChunkData;

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub seed: u64,
    pub chunks: Vec<ChunkData>,
    pub inventory: Inventory,
    pub score: Score,
    pub player_x: f32,
    pub player_y: f32,
}
