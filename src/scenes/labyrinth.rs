use crate::input;
use crate::level_configuration::{LevelConfiguration};
use crate::lighting::{TileLightTracing};
use crate::resources;
use crate::scenes;
use crate::types::{Point2, Rect, Vector2};
use crate::util;
use crate::world::World;
use ggez::audio::SoundSource;
use ggez::graphics;
use ggez::timer;
use ggez;
use ggez_goodies::scene;
use log::{debug, info, warn};
// use specs::{self, Join};
use std::f32::consts::PI;
use warmy;

const WALL_SIZE: f32 = 32.0;
const PLAYER_WIDTH: f32 = 16.0;
const PLAYER_HEIGHT: f32 = 16.0;
const PLAYER_MAX_SPEED: f32 = 5.0 * 60.0;
const PLAYER_MAX_ACCELERATION: f32 = 5.0 * 60.0;
const PLAYER_ACCELERATION_CONST: f32 = 2.0 * 60.0;
const PLAYER_COLLISION_STEPS: usize = 4;
const PLAYER_LIGHT_RADIUS: f32 = 100.0;

const RAY_COUNT: usize = 360;
const STEP_DISTANCE: f32 = 8.0;

pub struct LabyrinthScene {
	quit: bool,

	level: warmy::Res<resources::Level>,
	level_configuration: LevelConfiguration,

	player_image: warmy::Res<resources::Image>,

	tiles: resources::TilePack,

	player_coords: Point2,
	player_speed: f32,
	player_acceleration: f32,
	player_direction: Vector2,
	player_light_radius: f32,

	shards_collected: usize,
	are_doors_activated: bool,
	entered_door: bool,
	entities_visibility: Vec<bool>,

	dispatcher: specs::Dispatcher<'static, 'static>,
}

impl LabyrinthScene {
	pub fn new(world: &mut World, context: &mut ggez::Context, level_name: &str) -> Self {
		// TODO: Don't use paths here.

		let level = world.resources
			.get::<resources::Level>(&resources::ResourceKey::from_path(&format!("/levels/{}.toml", level_name)), context)
			.unwrap();
		let level_configuration = LevelConfiguration::new(&level.borrow(), resources::TILE_COUNT, resources::CORNER_COUNT);

		let player_image = world.resources
			.get::<resources::Image>(&resources::ResourceKey::from_path("/images/character-16x16.png"), context)
			.unwrap();

		let tiles = resources::TilePack::load(world, context, &level.borrow().key);
		let offset = level.borrow().get_offset(world.center(), (WALL_SIZE, WALL_SIZE));

		let player_coords = Point2::new(
			level.borrow().player_x * WALL_SIZE + offset.x,
			level.borrow().player_y * WALL_SIZE + offset.y,
		);
		let player_light_radius = level.borrow().player_light_radius;

		let are_doors_activated = level.borrow().shards_for_door_activation == 0;

		let entities_visibility = {
			let mut ret = Vec::new();

			for _ in level.borrow().entities.iter() {
				ret.push(true);
			}

			ret
		};

		let mut dispatcher = Self::register_systems();

		Self {
			quit: false,

			level,
			level_configuration,

			player_image,

			tiles,

			player_coords,
			player_speed: 0.0,
			player_acceleration: 0.0,
			player_direction: Vector2::zero(),
			player_light_radius,

			shards_collected: 0,
			are_doors_activated,
			entered_door: false,
			entities_visibility,

			dispatcher,
		}
	}

	fn register_systems() -> specs::Dispatcher<'static, 'static> {
		specs::DispatcherBuilder::new()
			// .with(MovementSystem, "sys_movement", &[])
			.build()
	}

	fn get_level_offset(&self, world: &mut World) -> Point2 {
		self.level.borrow().get_offset(world.center(), (WALL_SIZE, WALL_SIZE))
	}

	fn draw_light(&self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		// select tiles that are in player's radius
		let mut target_tiles = {
			let offset = self.get_level_offset(world);
			let level = &self.level.borrow();

			let mut tiles = Vec::new();

			for i in 0..level.width {
				for j in 0..level.height {
					if level.get(i, j).is_door() {
						if self.are_doors_activated {
							continue;
						}
					}
					else if level.get(i, j).is_empty() {
						continue;
					}

					let tile_id = j * level.width + i;

					let tile_position = Point2::new(
						offset.x + i as f32 * WALL_SIZE + WALL_SIZE / 2.0,
						offset.y + j as f32 * WALL_SIZE + WALL_SIZE / 2.0,
					);

					let distance = util::get_distance(tile_position, self.player_coords);

					if distance <= self.player_light_radius + WALL_SIZE {
						tiles.push(TileLightTracing::new(tile_id, tile_position, WALL_SIZE, WALL_SIZE));
					}
				}
			}

			tiles
		};

		// render all tiles in radius, for test
		if false {
			for tile in target_tiles.iter() {
				graphics::draw(
					context,
					&self.tiles.tile_up[0].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(tile.rect.x, tile.rect.y))
				)?;

				graphics::draw(
					context,
					&self.tiles.tile_down[0].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(tile.rect.x, tile.rect.y))
				)?;
			}
		}

		// assume player's position as 0,0 for all tiles
		TileLightTracing::set_origin(&mut target_tiles, self.player_coords);

		// start ray tracing
		use euclid::Angle;

		let mut degree = 0.0;

		for _ in 0..RAY_COUNT {
			let (sin, cos) = Angle::degrees(degree).sin_cos();
			let direction_v = Vector2::new(cos, sin);

			let mut step_n = 1;
			loop {
				let current_position = direction_v * STEP_DISTANCE * step_n as f32;
				if current_position.length() > self.player_light_radius {
					break;
				}

				if let Some(tile) = TileLightTracing::find_intersection_mut(&mut target_tiles, current_position.to_point()) {
					tile.register_hit(current_position.to_point());
					break;
				}

				step_n += 1;
			}

			degree += 360.0 / RAY_COUNT as f32;
		}

		// update coordinates of all tiles
		TileLightTracing::set_origin(&mut target_tiles, Point2::new(-self.player_coords.x, -self.player_coords.y));

		// render
		if false {
			for tile in target_tiles.iter() {
				if tile.hits > 0 {
					graphics::draw(
						context,
						&self.tiles.tile_up[0].borrow().0,
						graphics::DrawParam::default()
							.dest(Point2::new(tile.rect.x, tile.rect.y))
					)?;

					graphics::draw(
						context,
						&self.tiles.tile_down[0].borrow().0,
						graphics::DrawParam::default()
							.dest(Point2::new(tile.rect.x, tile.rect.y))
					)?;
				}
			}
		}

		if true {
			for tile in target_tiles.iter() {
				tile.draw(context, &self.tiles, &self.level_configuration)?;
			}
		}

		Ok(())
	}

	fn draw_player(&self, context: &mut ggez::Context) -> ggez::GameResult<()> {
		let player = &self.player_image.borrow().0;

		let x = self.player_coords.x - PLAYER_WIDTH as f32 / 2.0;
		let y = self.player_coords.y - PLAYER_HEIGHT as f32 / 2.0;

		graphics::draw(
			context,
			player,
			graphics::DrawParam::default()
				.dest(Point2::new(x, y))
		)?;

		Ok(())
	}

	fn get_tile_id(&self, world: &mut World, screen_coords: Point2) -> Option<usize> {
		let offset = self.get_level_offset(world);
		let point = screen_coords - offset;

		let x = (point.x / WALL_SIZE) as isize;
		let y = (point.y / WALL_SIZE) as isize;

		let level = &self.level.borrow();
		let tile_id = level.width as isize * y + x;

		if tile_id >= 0 && tile_id < level.walls.len() as isize {
			Some(tile_id as usize)
		}
		else {
			None
		}
	}

	fn get_tile(&self, tile_id: usize) -> resources::Wall {
		self.level.borrow().walls[tile_id].clone()
	}

	fn move_player(&mut self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		let direction = self.player_direction;

		let dt = timer::duration_to_f64(timer::delta(context)) as f32;
		// let fps = 1000 / timer::average_delta(context).as_millis();
		// info!("fps: {}, dt: {}", fps, dt);

		if direction.length() > 0.0 {
			// moving
			self.player_acceleration += PLAYER_ACCELERATION_CONST;

			if self.player_acceleration > PLAYER_MAX_ACCELERATION {
				self.player_acceleration = PLAYER_MAX_ACCELERATION;
			}

			self.player_speed += self.player_acceleration;

			if self.player_speed > PLAYER_MAX_SPEED {
				self.player_speed = PLAYER_MAX_SPEED;
			}

			let movement_v = Vector2::new(direction.x * self.player_speed, direction.y * self.player_speed) * dt;

			self.move_player_with_collisions(world, context, movement_v)?;
		}
		else {
			// not moving
			// TODO: slowing down?

			self.player_acceleration = 0.0;
			self.player_speed = 0.0;
		}

		Ok(())
	}

	fn pick_up_items(&mut self, world: &mut World, _context: &mut ggez::Context) -> ggez::GameResult<()> {
		let offset = self.get_level_offset(world);
		let level = &self.level.borrow();

		for (index, entity) in level.entities.iter().enumerate() {
			if !self.entities_visibility[index] {
				continue;
			}

			let position = Point2::new(
				entity.x * WALL_SIZE + offset.x,
				entity.y * WALL_SIZE + offset.y,
			);

			let distance = util::get_distance(position, self.player_coords);

			if distance < (WALL_SIZE + PLAYER_WIDTH) / 2.0 {
				match entity.effect {
					resources::PickUpEffect::IncreasePlayerLightRadius => {
						self.player_light_radius = PLAYER_LIGHT_RADIUS;
					},
					resources::PickUpEffect::ActivateDoors => {
						self.shards_collected += 1;

						if self.shards_collected >= level.shards_for_door_activation {
							self.are_doors_activated = true;
						}
					},
				}

				self.entities_visibility[index] = false;
				let _ = world.sound_pick_up.play_detached();
			}
		}

		Ok(())
	}

	fn move_player_with_collisions(&mut self, world: &mut World, context: &mut ggez::Context, movement_v: Vector2) -> ggez::GameResult<()> {
		let mut current = Rect::new(
			self.player_coords.x - PLAYER_WIDTH / 2.0,
			self.player_coords.y - PLAYER_HEIGHT / 2.0,
			PLAYER_WIDTH,
			PLAYER_HEIGHT,
		);

		if movement_v.x != 0.0 {
			let mut possible = current;

			for _ in 0..PLAYER_COLLISION_STEPS {
				possible.x += movement_v.x / PLAYER_COLLISION_STEPS as f32;

				if !self.check_wall_collision(world, context, possible) {
					current.x = possible.x;
				}
			}
		}

		if movement_v.y != 0.0 {
			let mut possible = current;

			for _ in 0..PLAYER_COLLISION_STEPS {
				possible.y += movement_v.y / PLAYER_COLLISION_STEPS as f32;

				if !self.check_wall_collision(world, context, possible) {
					current.y = possible.y;
				}
			}
		}

		self.player_coords.x = current.x + PLAYER_WIDTH / 2.0;
		self.player_coords.y = current.y + PLAYER_HEIGHT / 2.0;

		Ok(())
	}

	fn check_wall_collision(&mut self, world: &mut World, context: &mut ggez::Context, object: Rect) -> bool {
		let o = object;

		let points = vec![
			Point2::new(o.x, o.y),
			Point2::new(o.x + o.w, o.y),
			Point2::new(o.x + o.w, o.y + o.h),
			Point2::new(o.x, o.y + o.h),
		];

		for point in points.iter() {
			if let Some(wall) = self.get_tile_by_point(world, context, *point) {
				if wall.is_door() {
					if self.are_doors_activated {
						self.entered_door = true;
						let _ = world.sound_door.play_detached();
					}
					else {
						return true;
					}
				}
				else if wall.is_wall() {
					return true;
				}
			}
		}

		false
	}

	fn get_tile_by_point(&mut self, world: &mut World, _context: &mut ggez::Context, point: Point2) -> Option<resources::Wall> {
		if let Some(tile_id) = self.get_tile_id(world, point) {
			Some(self.get_tile(tile_id))
		}
		else {
			None
		}
	}

	fn draw_doors(&self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		if !self.are_doors_activated {
			return Ok(());
		}

		let offset = self.get_level_offset(world);
		let level = &self.level.borrow();

		for i in 0..level.width {
			for j in 0..level.height {
				let tile = level.get(i, j);
				if !tile.is_door() {
					continue;
				}

				let position = Point2::new(
					offset.x + i as f32 * WALL_SIZE + WALL_SIZE / 2.0,
					offset.y + j as f32 * WALL_SIZE + WALL_SIZE / 2.0,
				);

				let (rotate, image) = match tile {
					resources::Wall::B0H => ( 0.0, &self.tiles.door_2_0),
					resources::Wall::B1H => ( 0.0, &self.tiles.door_2_1),
					resources::Wall::B0V => (90.0, &self.tiles.door_2_0),
					resources::Wall::B1V => (90.0, &self.tiles.door_2_1),
					resources::Wall::D0H => ( 0.0, &self.tiles.door_3_0),
					resources::Wall::D1H => ( 0.0, &self.tiles.door_3_1),
					resources::Wall::D2H => ( 0.0, &self.tiles.door_3_2),
					resources::Wall::D0V => (90.0, &self.tiles.door_3_0),
					resources::Wall::D1V => (90.0, &self.tiles.door_3_1),
					resources::Wall::D2V => (90.0, &self.tiles.door_3_2),
					_ => panic!("tile {:?} isn't a door!", tile),
				};

				graphics::draw(
					context,
					&image.borrow().0,
					graphics::DrawParam::default()
						.dest(position)
						.rotation(rotate * PI / 180.0)
						.offset(Point2::new(0.5, 0.5))
				)?;
			}
		}

		Ok(())
	}

	fn draw_shards(&self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		let offset = self.get_level_offset(world);
		let level = &self.level.borrow();

		for (index, entity) in level.entities.iter().enumerate() {
			if !self.entities_visibility[index] {
				continue;
			}

			let position = Point2::new(
				entity.x * WALL_SIZE + offset.x,
				entity.y * WALL_SIZE + offset.y,
			);

			let distance = util::get_distance(position, self.player_coords);

			if distance <= self.player_light_radius || distance <= entity.light_radius {
				let image = match entity.entity_type {
					resources::EntityType::Shard0 => &self.tiles.shard_0,
					resources::EntityType::Shard1 => &self.tiles.shard_1,
					resources::EntityType::Shard2 => &self.tiles.shard_2,
					resources::EntityType::Shard3 => &self.tiles.shard_3,
					resources::EntityType::Shard4 => &self.tiles.shard_4,
				};

				graphics::draw(
					context,
					&image.borrow().0,
					graphics::DrawParam::default()
						.dest(position)
						.offset(Point2::new(0.5, 0.5))
				)?;
			}
		}

		Ok(())
	}

	// fn check_wall_collision_old(&mut self, _world: &mut World, context: &mut ggez::Context, object: Rect) -> ggez::GameResult<bool> {
	// 	let offset = self.get_level_offset(context);

	// 	if let Some(tile_id) = self.get_tile_id(context, center(&object)) {
	// 		info!("player is on a tile {}", tile_id);

	// 		for neighbor_n in 0..8 {
	// 			if let Some(neighbor_id) = self.get_neighbor_id(tile_id, neighbor_n) {
	// 				info!("checking neighbor {}:{}", neighbor_n, neighbor_id);

	// 				let mut tile_rect = self.get_tile_rect(neighbor_id);
	// 				tile_rect.x += offset.x;
	// 				tile_rect.y += offset.y;

	// 				if self.get_tile(neighbor_id) != resources::Wall::N
	// 					&& util::check_collision(object, tile_rect)
	// 				{
	// 					info!("player at tile_id {} collided with neighbor {}:{}", tile_id, neighbor_n, neighbor_id);

	// 					return Ok(true);
	// 				}
	// 			}
	// 		}
	// 	}
	// 	else {
	// 		info!("player isn't in a labyrinth");
	// 	}

	// 	Ok(false)
	// }

	// fn move_player(&mut self, _world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
	// 	if self.player_coords == self.player_coords_next {
	// 		return Ok(());
	// 	}

	// 	let next = self.player_coords_next;
	// 	let player_rect = Rect::new(
	// 		next.x - PLAYER_SIZE / 2.0,
	// 		next.y - PLAYER_SIZE / 2.0,
	// 		PLAYER_SIZE,
	// 		PLAYER_SIZE
	// 	);
	// 	let offset = self.get_level_offset(context);

	// 	if let Some(tile_id) = self.get_tile_id(context, self.player_coords_next) {
	// 		// info!("player is on a tile {}", tile_id);

	// 		for neighbor_n in 0..8 {
	// 			if let Some(neighbor_id) = self.get_neighbor_id(tile_id, neighbor_n) {
	// 				// info!("checking neighbor {}:{}", neighbor_n, neighbor_id);

	// 				let mut tile_rect = self.get_tile_rect(neighbor_id);
	// 				tile_rect.x += offset.x;
	// 				tile_rect.y += offset.y;

	// 				if self.get_tile(neighbor_id) == resources::Wall::Solid
	// 					&& util::check_collision(player_rect, tile_rect)
	// 				{
	// 					// info!("player at tile_id {} collided with neighbor {}:{}", tile_id, neighbor_n, neighbor_id);

	// 					return Ok(());
	// 				}
	// 			}
	// 		}
	// 	}
	// 	else {
	// 		info!("player isn't in a labyrinth");
	// 	}

	// 	// no collisions, allowing the move
	// 	self.player_coords = self.player_coords_next;

	// 	Ok(())
	// }
}

impl scene::Scene<World, input::Event> for LabyrinthScene {
	fn update(&mut self, world: &mut World, context: &mut ggez::Context) -> scenes::Switch {
		self.dispatcher.dispatch(&mut world.specs_world);

		self.move_player(world, context)
			.expect("Failed to move player...");
		self.pick_up_items(world, context)
			.expect("Failed to pick up items...");

		if self.quit {
			scene::SceneSwitch::Pop
		}
		else if self.entered_door {
			world.next_scene(context)
		}
		else {
			scene::SceneSwitch::None
		}
	}

	fn draw(&mut self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		// self.draw_level(world, context)?;
		self.draw_light(world, context)?;
		self.draw_doors(world, context)?;
		self.draw_shards(world, context)?;
		self.draw_player(context)?;

		Ok(())
	}

	fn name(&self) -> &str {
		"LabyrinthScene"
	}

	fn input(&mut self, world: &mut World, _ev: input::Event, _started: bool) {
		if world.input.get_button_pressed(input::Button::Quit) {
			info!("pressed quit");
			self.quit = true;
		}

		// self.player_coords.x += world.input.get_axis(input::Axis::Horz);
		// self.player_coords.y -= world.input.get_axis(input::Axis::Vert);

		self.player_direction = Vector2::zero();

		if world.input.get_button_down(input::Button::Left) {
			self.player_direction.x -= 1.0;
		}
		if world.input.get_button_down(input::Button::Right) {
			self.player_direction.x += 1.0;
		}
		if world.input.get_button_down(input::Button::Up) {
			self.player_direction.y -= 1.0;
		}
		if world.input.get_button_down(input::Button::Down) {
			self.player_direction.y += 1.0;
		}

		self.player_direction = self.player_direction.normalize();

		//

		// self.player_coords_next = self.player_coords;

		// if world.input.get_button_down(input::Button::Left) {
		// 	self.player_coords_next.x -= max_speed;
		// }
		// if world.input.get_button_down(input::Button::Right) {
		// 	self.player_coords_next.x += max_speed;
		// }
		// if world.input.get_button_down(input::Button::Up) {
		// 	self.player_coords_next.y -= max_speed;
		// }
		// if world.input.get_button_down(input::Button::Down) {
		// 	self.player_coords_next.y += max_speed;
		// }

		// // new Point2
		// // try moving
	}
}
