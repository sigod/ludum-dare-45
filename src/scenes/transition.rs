use crate::input;
use crate::resources;
use crate::scenes;
use crate::types::{Point2};
use crate::world::World;
use ggez::graphics;
use ggez::timer;
use ggez;
use ggez_goodies::scene;
use serde::{Deserialize};
use std::path;

#[derive(Clone, Debug, Deserialize)]
struct AnimatedScreenInfo {
	timing: f32,
	total_time: bool,
	folder_name: String,
	image_count: usize,
}

impl AnimatedScreenInfo {
	fn load<P: AsRef<path::Path>>(context: &mut ggez::Context, file: P) -> ggez::GameResult<Self> {
		use std::io::Read;

		let mut content = String::new();
		let mut reader = ggez::filesystem::open(context, file)?;
		let _ = reader.read_to_string(&mut content)?;

		let list: Self = toml::from_str(&content)
			.map_err(|e| ggez::error::GameError::ResourceLoadError(e.to_string()))?;

		Ok(list)
	}
}

struct AnimatedScreen {
	images: Vec<warmy::Res<resources::Image>>,
	timing: f32,
}

impl AnimatedScreen {
	fn load<P: AsRef<path::Path>>(world: &mut World, context: &mut ggez::Context, file: P) -> ggez::GameResult<Option<Self>> {
		if !ggez::filesystem::exists(context, &file) {
			return Ok(None)
		}

		let info = AnimatedScreenInfo::load(context, &file)?;

		let timing = if info.total_time {
			info.timing / info.image_count as f32
		}
		else {
			info.timing
		};

		let mut images = Vec::new();

		for index in 0..info.image_count {
			let image_path = format!("/images/animated/{}/{:02}.png", info.folder_name, index);

			let image = world.resources
				.get::<resources::Image>(&resources::ResourceKey::from_path(&image_path), context)
				.unwrap();
			images.push(image);
		}

		let ret = Self {
			images,
			timing,
		};

		Ok(Some(ret))
	}
}

enum SceneType {
	Static(warmy::Res<resources::Image>),
	Animated(AnimatedScreen),
}

pub struct TransitionScene {
	is_main: bool,

	scene: SceneType,
	current_image: usize,
	extra_dt: f32,

	should_switch_next: bool,
	should_quit: bool,
}

impl TransitionScene {
	pub fn new(world: &mut World, context: &mut ggez::Context, is_main: bool, screen: &str) -> Self {
		let animated = AnimatedScreen::load(world, context, &format!("/animated/{}.toml", screen))
			.expect("Unable to load animated screen!");

		let scene = if let Some(animated) = animated {
			SceneType::Animated(animated)
		}
		else {
			let image = world.resources
				.get::<resources::Image>(&resources::ResourceKey::from_path(&format!("/images/transitions/{}.png", screen)), context)
				.unwrap();

			SceneType::Static(image)
		};

		Self {
			is_main,

			scene,
			current_image: 0,
			extra_dt: 0.0,

			should_switch_next: false,
			should_quit: false,
		}
	}

	fn update_frame(&mut self, dt: f32) {
		if let SceneType::Animated(animated) = &self.scene {
			self.extra_dt += dt;

			while self.extra_dt > animated.timing {
				if self.extra_dt - animated.timing < 0.0 {
					break;
				}

				self.extra_dt -= animated.timing;

				self.current_image += 1;
				if self.current_image == animated.images.len() {
					self.current_image = 0;
				}
			}
		}
	}

	fn current(&self) -> &warmy::Res<resources::Image> {
		if let SceneType::Animated(animated) = &self.scene {
			&animated.images[self.current_image]
		}
		else {
			panic!("Transition is not animated!");
		}
	}

	fn is_animated(&self) -> bool {
		if let SceneType::Animated(_animated) = &self.scene {
			true
		}
		else {
			false
		}
	}
}

impl scene::Scene<World, input::Event> for TransitionScene {
	fn update(&mut self, world: &mut World, context: &mut ggez::Context) -> scenes::Switch {
		if self.should_switch_next {
			self.should_switch_next = false;

			world.next_scene(context)
		}
		else if self.should_quit {
			self.should_quit = false;

			if self.is_main {
				world.exit = true;

				scene::SceneSwitch::None
			}
			else {
				world.reset_scenes();

				scene::SceneSwitch::Pop
			}
		}
		else {
			scene::SceneSwitch::None
		}
	}

	fn draw(&mut self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		let position = world.center();

		let dt = timer::duration_to_f64(timer::delta(context)) as f32;
		self.update_frame(dt);

		if self.is_animated() {
			if let SceneType::Animated(_animated) = &self.scene {
				graphics::draw(
					context,
					&self.current().borrow().0,
					graphics::DrawParam::default()
						.dest(position)
						.offset(Point2::new(0.5, 0.5))
				)?;
			}
			else {
				panic!("Transition is not animated!");
			}
		}
		else {
			if let SceneType::Static(image) = &self.scene {
				graphics::draw(
					context,
					&image.borrow().0,
					graphics::DrawParam::default()
						.dest(position)
						.offset(Point2::new(0.5, 0.5))
				)?;
			}
			else {
				panic!("Transition is not static!");
			}
		}

		Ok(())
	}

	fn name(&self) -> &str {
		"TransitionScene"
	}

	fn input(&mut self, world: &mut World, _ev: input::Event, _started: bool) {
		if world.input.get_button_pressed(input::Button::Quit) {
			self.should_quit = true;
		}
		if world.input.get_button_pressed(input::Button::Next) {
			self.should_switch_next = true;
		}
	}
}
