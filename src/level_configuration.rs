use crate::resources::{Level};
use rand::{thread_rng, Rng};

pub struct TileConfiguration {
	pub tile_id: usize,
	pub side: usize,
	pub corner: usize,
}

pub struct LevelConfiguration {
	pub tiles: Vec<TileConfiguration>,
}

impl LevelConfiguration {
	pub fn new(level: &Level, side_count: usize, corner_count: usize) -> Self {
		let mut tiles = Vec::new();
		let mut rng = thread_rng();

		for (tile_id, _) in level.walls.iter().enumerate() {
			tiles.push(TileConfiguration {
				tile_id,
				side: rng.gen_range(0, side_count),
				corner: rng.gen_range(0, corner_count),
			});
		}

		Self {
			tiles,
		}
	}

	pub fn get_side(&self, tile_id: usize) -> usize {
		assert!(tile_id < self.tiles.len(), "Invalid tile_id {}!", tile_id);

		self.tiles[tile_id].side
	}

	pub fn get_corner(&self, tile_id: usize) -> usize {
		assert!(tile_id < self.tiles.len(), "Invalid tile_id {}!", tile_id);

		self.tiles[tile_id].corner
	}
}
