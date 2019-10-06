use crate::types::Point2;
use crate::{components, resources, input};
use specs::prelude::*;
use specs::{self};
use std::path;
use warmy;

pub struct World {
	pub resources: resources::Store,
	pub input: input::State,
	pub specs_world: specs::World,
	pub exit: bool,
	pub dimensions: (f32, f32),
}

impl World {
	pub fn new(resource_path: &path::Path, dimensions: (f32, f32)) -> Self {
		// TODO: There are potential problems.
		// See https://github.com/ggez/game-template/blob/master/src/world.rs
		let opt = warmy::StoreOpt::default().set_root(resource_path);
		let store = warmy::Store::new(opt)
			.expect("Could not create asset store? Does the directory exist?");

		let mut specs_world = specs::World::new();
		components::register_components(&mut specs_world);

		let world = Self {
			resources: store,
			input: input::State::new(),
			specs_world,
			exit: false,
			dimensions,
		};

		// TODO: Make a player, for a test?

		world
	}

	pub fn center(&self) -> Point2 {
		Point2::new(self.dimensions.0 / 2.0, self.dimensions.1 / 2.0)
	}
}
