#[macro_use]
extern crate log;

use std::env;
use std::path;
use ggez::{self, *};

mod components;
mod input;
mod resources;
mod scenes;
mod systems;
mod types;
mod util;
mod world;

const DESIRED_FPS: u32 = 60;
const DIMENSIONS: (f32, f32) = (1920.0, 1080.0);
const GAME_ID: &str = "LD45";
const GAME_TITLE: &str = "LD45";
const AUTHOR: &str = "PickleMagicle";

struct MainState {
	scenes: scenes::Stack,
	input_binding: input::Binding,
}

impl MainState {
	fn new(context: &mut Context, resource_path: &path::Path) -> Self {
		let world = world::World::new(resource_path, DIMENSIONS);
		let mut scenes = scenes::Stack::new(context, world);
		let initial_scene = Box::new(scenes::main_menu::MainMenuScene::new(context, &mut scenes.world));
		scenes.push(initial_scene);

		Self {
			scenes,
			input_binding: input::create_input_binding(),
		}
	}
}

impl event::EventHandler for MainState {
	fn update(&mut self, context: &mut Context) -> GameResult<()> {
		while timer::check_update_time(context, DESIRED_FPS) {
			self.scenes.update(context);
		}
		self.scenes.world.resources.sync(context);
		self.scenes.world.input.update(timer::duration_to_f64(timer::delta(context)) as f32);

		if self.scenes.world.exit {
			info!("Exiting due to world quit flag.");
            event::quit(context);
		}

		Ok(())
	}

	fn draw(&mut self, context: &mut Context) -> GameResult<()> {
		graphics::clear(context, graphics::Color::from((0.0, 0.0, 0.0, 0.0)));
		self.scenes.draw(context);
		graphics::present(context)
	}

	fn key_down_event(
		&mut self,
		_ctx: &mut Context,
		keycode: event::KeyCode,
		_keymod: event::KeyMods,
		_repeat: bool,
	) {
		if let Some(ev) = self.input_binding.resolve(keycode) {
			self.scenes.world.input.update_effect(ev, true);
			self.scenes.input(ev, true);
			// TODO: update_button_down?
		}
	}

	fn key_up_event(
		&mut self,
		_ctx: &mut Context,
		keycode: event::KeyCode,
		_keymod: event::KeyMods,
	) {
		if let Some(ev) = self.input_binding.resolve(keycode) {
			self.scenes.world.input.update_effect(ev, false);
			self.scenes.input(ev, false);
			// TODO: update_button_up?
		}
	}

	fn resize_event(&mut self, context: &mut Context, width: f32, height: f32) {
		info!("received resize event: {}x{}", width, height);
		info!("screen_coordinates: {:?}", graphics::screen_coordinates(context));

		// graphics::set_screen_coordinates(context, graphics::Rect::new(
		// 		(DIMENSIONS.0 - width) / 2.0,
		// 		(DIMENSIONS.1 - height) / 2.0,
		// 		width + (DIMENSIONS.0 - width) / 2.0,
		// 		height + (DIMENSIONS.1 - height) / 2.0,
		// 	))
		// 	.expect("set_screen_coordinates failed!");

		// graphics::set_screen_coordinates(context, graphics::Rect::new(0.0, 0.0, width, height))
		// 	.expect("set_screen_coordinates failed!");

		// graphics::set_screen_coordinates(context, graphics::Rect::new(0.0, 0.0, width * 1.25, height * 1.25))
		// 	.expect("set_screen_coordinates failed!");

		graphics::set_screen_coordinates(context, graphics::Rect::new(0.0, 0.0, DIMENSIONS.0, DIMENSIONS.1))
			.expect("set_screen_coordinates failed!");

		info!("screen_coordinates: {:?}", graphics::screen_coordinates(context));
	}
}

fn main() {
	util::setup_logging();

	let resource_path = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
		let mut path = path::PathBuf::from(manifest_dir);
		path.push("resources");

		path
	}
	else {
		path::PathBuf::from("./resources")
	};
	info!("Resource path: {:?}", resource_path);

	let cb = ContextBuilder::new(GAME_ID, AUTHOR)
		.window_setup(conf::WindowSetup::default().title(GAME_TITLE))
		.window_mode(conf::WindowMode::default()
			.dimensions(DIMENSIONS.0, DIMENSIONS.1)
			// .min_dimensions(DIMENSIONS.0, DIMENSIONS.1)
			.fullscreen_type(conf::FullscreenType::Desktop)
			.borderless(true)
			.maximized(true)
		)
		.add_resource_path(&resource_path);
	let (context, ev) = &mut cb.build().unwrap();

	// graphics::set_blend_mode(context, graphics::BlendMode::Alpha).unwrap();

	info!("main: screen_coordinates: {:?}", graphics::screen_coordinates(context));
	// TODO: Fix scale issue, try https://docs.rs/ggez/0.5.1/ggez/graphics/fn.set_screen_coordinates.html

	let state = &mut MainState::new(context, &resource_path);
	if let Err(e) = event::run(context, ev, state) {
		error!("Error encountered: {}", e);
	}
	else {
		info!("Game exited cleanly.");
	}
}
