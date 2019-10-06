use crate::components as c;
use crate::input;
use crate::resources;
use crate::scenes;
use crate::systems::*;
use crate::types;
use crate::util;
use crate::world::World;
use ggez::graphics;
use ggez;
use ggez_goodies::scene;
use log::*;
use specs::{self, Join};
use warmy;

// TODO: Simplify main menu. We don't need dispatcher and other complicated logic.

pub struct MainMenuScene {
	start: bool,
	screen: warmy::Res<resources::Image>,
	dispatcher: specs::Dispatcher<'static, 'static>,
}

impl MainMenuScene {
	pub fn new(context: &mut ggez::Context, world: &mut World) -> Self {
		let screen = world.resources
			// TODO: Don't use path here...
			.get::<resources::Image>(&resources::ResourceKey::from_path("/images/main_menu_screen.png"), context)
			.unwrap();
		let mut dispatcher = Self::register_systems();

		Self {
			start: false,
			screen,
			dispatcher,
		}
	}

	fn register_systems() -> specs::Dispatcher<'static, 'static> {
		specs::DispatcherBuilder::new()
			// .with(MovementSystem, "sys_movement", &[])
			.build()
	}
}

impl scene::Scene<World, input::Event> for MainMenuScene {
	fn update(&mut self, world: &mut World, context: &mut ggez::Context) -> scenes::Switch {
		self.dispatcher.dispatch(&mut world.specs_world);

		if self.start {
			use crate::scenes::labyrinth::LabyrinthScene;

			self.start = false;

			scene::SceneSwitch::push(LabyrinthScene::new(context, world))
		}
		else {
			scene::SceneSwitch::None
		}
	}

	fn draw(&mut self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		// let position = world.specs_world.read_storage::<c::Position>();
		// for p in position.join() {
		// 	graphics::draw(context, &(self.kiwi.borrow().0), graphics::DrawParam::default().dest(p.0))?;
		// }

		// // let drawable_size = graphics::drawable_size(context);
		// // info!("drawable_size: {:?}", drawable_size);
		// let drawable_size = (1920.0, 1080.0);
		let image = &self.screen.borrow().0;

		let x = (world.dimensions.0 - image.width() as f32) / 2.0;
		let y = (world.dimensions.1 - image.height() as f32) / 2.0;

		graphics::draw(
			context,
			image,
			graphics::DrawParam::default()
				.dest(types::Point2::new(x, y)))?;

		Ok(())
	}

	fn name(&self) -> &str {
		"MainMenuScene"
	}

	fn input(&mut self, world: &mut World, ev: input::Event, _started: bool) {
		if world.input.get_button_pressed(input::Button::Quit) {
			info!("pressed quit");
			world.exit = true;
		}
		if world.input.get_button_pressed(input::Button::Next) {
			info!("pressed start");
			self.start = true;
		}
	}
}
