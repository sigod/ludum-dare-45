use crate::input;
use crate::resources;
use crate::scenes;
use crate::types::{Point2};
use crate::world::World;
use ggez::graphics;
use ggez;
use ggez_goodies::scene;

pub struct TransitionScene {
	is_main: bool,

	image: warmy::Res<resources::Image>,

	should_switch_next: bool,
	should_quit: bool,
}

impl TransitionScene {
	pub fn new(world: &mut World, context: &mut ggez::Context, is_main: bool, screen: &str) -> Self {
		let image = world.resources
			.get::<resources::Image>(&resources::ResourceKey::from_path(&format!("/images/transitions/{}.png", screen)), context)
			.unwrap();

		Self {
			is_main,
			image,
			should_switch_next: false,
			should_quit: false,
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

		graphics::draw(
			context,
			&self.image.borrow().0,
			graphics::DrawParam::default()
				.dest(position)
				.offset(Point2::new(0.5, 0.5))
		)?;

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
