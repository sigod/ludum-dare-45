use crate::resources::{TransitionList, TransitionType};
use crate::scenes::labyrinth::LabyrinthScene;
use crate::scenes::transition::TransitionScene;
use crate::scenes;
use crate::types::Point2;
use crate::{components, resources, input};
use ggez::audio;
use ggez::{Context};
use ggez_goodies::scene;
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
	pub transition_list: TransitionList,

	pub sound_door: audio::Source,
	pub sound_pick_up: audio::Source,
	pub sound_background: audio::Source,
}

impl World {
	pub fn new(context: &mut Context, resource_path: &path::Path, transition_list: TransitionList, dimensions: (f32, f32)) -> Self {
		// TODO: There are potential problems.
		// See https://github.com/ggez/game-template/blob/master/src/world.rs
		let opt = warmy::StoreOpt::default().set_root(resource_path);
		let store = warmy::Store::new(opt)
			.expect("Could not create asset store? Does the directory exist?");

		let sound_door = audio::Source::new(context, "/audio/door.wav")
			.expect("Count not load door sound!");
		let sound_pick_up = audio::Source::new(context, "/audio/pick-up.wav")
			.expect("Count not load item pick up sound!");
		let sound_background = audio::Source::new(context, "/audio/background.mp3")
			.expect("Count not load background sound!");

		let mut specs_world = specs::World::new();
		components::register_components(&mut specs_world);

		let world = Self {
			resources: store,
			input: input::State::new(),
			specs_world,
			exit: false,
			dimensions,
			transition_list,

			sound_door,
			sound_pick_up,
			sound_background,
		};

		world
	}

	pub fn center(&self) -> Point2 {
		Point2::new(self.dimensions.0 / 2.0, self.dimensions.1 / 2.0)
	}

	pub fn initial_scene(&mut self, context: &mut ggez::Context) -> TransitionScene {
		let current = self.transition_list.transitions[0].clone();
		let is_main = true;

		let scene = match current.transition_type {
			TransitionType::ToScreen => {
				TransitionScene::new(self, context, is_main, &current.name)
			},
			TransitionType::ToLevel => panic!("First level expected to be a screen!"),
		};

		scene
	}

	pub fn next_scene(&mut self, context: &mut ggez::Context) -> scenes::Switch {
		if self.transition_list.current_n + 1 == self.transition_list.transitions.len() {
			self.reset_scenes();

			return scene::SceneSwitch::Pop;
		}

		let is_main = self.transition_list.current_n == 0;

		self.transition_list.current_n += 1;
		let current = self.transition_list.transitions[self.transition_list.current_n].clone();

		let switch = match current.transition_type {
			TransitionType::ToLevel => {
				let scene = LabyrinthScene::new(self, context, &current.name);

				if is_main {
					scene::SceneSwitch::push(scene)
				}
				else {
					scene::SceneSwitch::replace(scene)
				}
			},
			TransitionType::ToScreen => {
				let scene = TransitionScene::new(self, context, is_main, &current.name);

				if is_main {
					scene::SceneSwitch::push(scene)
				}
				else {
					scene::SceneSwitch::replace(scene)
				}
			},
		};

		switch
	}

	pub fn reset_scenes(&mut self) {
		self.transition_list.current_n = 0;
	}
}
