use crate::components as c;
use crate::input;
use crate::resources;
use crate::scenes;
use crate::systems::*;
use crate::types::{self, Point2, Rect, Vector2};
use crate::util;
use crate::world::World;
use ggez::graphics;
use ggez;
use ggez_goodies::scene;
use log::{debug, info, warn};
use specs::{self, Join};
use warmy;
use std::f32::consts::PI;

const WALL_SIZE: f32 = 32.0;
const PLAYER_WIDTH: f32 = 16.0;
const PLAYER_HEIGHT: f32 = 16.0;
const PLAYER_MAX_SPEED: f32 = 5.0;
const PLAYER_MAX_ACCELERATION: f32 = 5.0;
const PLAYER_ACCELERATION_CONST: f32 = 2.0;
const PLAYER_COLLISION_STEPS: usize = 4;

const RAY_COUNT: usize = 360;
const STEP_DISTANCE: f32 = 8.0;

pub struct LabyrinthScene {
	quit: bool,
	level: warmy::Res<resources::Level>,
	player_image: warmy::Res<resources::Image>,

	tiles: resources::TilePack,

	player_coords: Point2,
	player_speed: f32,
	player_acceleration: f32,
	player_direction: Vector2,
	player_light_radius: f32,

	dispatcher: specs::Dispatcher<'static, 'static>,
}

impl LabyrinthScene {
	pub fn new(context: &mut ggez::Context, world: &mut World) -> Self {
		// TODO: Don't use paths here.

		let level = world.resources
			.get::<resources::Level>(&resources::ResourceKey::from_path("/level-1.toml"), context)
			.unwrap();
		let player_image = world.resources
			.get::<resources::Image>(&resources::ResourceKey::from_path("/images/character-16x16.png"), context)
			.unwrap();

		let tiles = resources::TilePack::load(world, context, &level.borrow().key);

		let player_coords = world.center();
		let mut dispatcher = Self::register_systems();

		Self {
			quit: false,
			level,
			player_image,

			tiles,

			player_coords,
			player_speed: 0.0,
			player_acceleration: 0.0,
			player_direction: Vector2::zero(),
			player_light_radius: 100.0,

			dispatcher,
		}
	}

	fn register_systems() -> specs::Dispatcher<'static, 'static> {
		specs::DispatcherBuilder::new()
			// .with(MovementSystem, "sys_movement", &[])
			.build()
	}

	fn get_level_offset(&self, world: &mut World) -> Point2 {
		let screen_center = world.center();
		let level = &self.level.borrow();

		let level_width = level.width as f32 * WALL_SIZE;
		let level_height = level.height as f32 * WALL_SIZE;

		let x = screen_center.x - level_width / 2.0;
		let y = screen_center.y - level_height / 2.0;

		Point2::new(x, y)
	}

	fn draw_level(&self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		let offset = self.get_level_offset(world);
		let level = &self.level.borrow();

		for i in 0..level.width {
			for j in 0..level.height {
				match level.get(i, j) {
					resources::Wall::N => {},
					resources::Wall::S => {
						let x = offset.x + i as f32 * WALL_SIZE;
						let y = offset.y + j as f32 * WALL_SIZE;

						graphics::draw(
							context,
							&self.tiles.tile_up[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(x, y))
						)?;

						graphics::draw(
							context,
							&self.tiles.tile_down[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(x, y))
						)?;
					},
				};
			}
		}

		Ok(())
	}

	fn draw_light(&self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		const SEGMENT_COUNT: usize = 8;

		#[derive(Debug)]
		struct SegmentPalette {
			palette: [bool; SEGMENT_COUNT],
		}

		impl SegmentPalette {
			fn new(segment_hits: &[usize; SEGMENT_COUNT]) -> Self {
				let palette: [bool; SEGMENT_COUNT] =
				[
					// segment_hits[0] > 0, segment_hits[1] > 0, segment_hits[2] > 0,
					// segment_hits[7] > 0,                      segment_hits[3] > 0,
					// segment_hits[6] > 0, segment_hits[5] > 0, segment_hits[4] > 0,

					segment_hits[0] > 0,
					segment_hits[1] > 0,
					segment_hits[2] > 0,
					segment_hits[3] > 0,
					segment_hits[4] > 0,
					segment_hits[5] > 0,
					segment_hits[6] > 0,
					segment_hits[7] > 0,
				];

				Self {
					palette,
				}
			}

			fn is_up(&self) -> bool {
				let p = self.palette;

				false
					|| (p[0] && p[1] && !p[2])
					|| (p[0] && !p[1] && p[2])
					|| (!p[0] && p[1] && !p[2])
					|| (!p[0] && p[1] && p[2])
					|| (p[0] && p[1] && p[2])
			}

			fn is_right(&self) -> bool {
				let p = self.palette;

				false
					|| (p[2] && p[3] && !p[4])
					|| (p[2] && !p[3] && p[4])
					|| (!p[2] && p[3] && !p[4])
					|| (!p[2] && p[3] && p[4])
					|| (p[2] && p[3] && p[4])
			}

			fn is_down(&self) -> bool {
				let p = self.palette;

				false
					|| (p[6] && p[5] && !p[4])
					|| (p[6] && !p[5] && p[4])
					|| (!p[6] && p[5] && !p[4])
					|| (!p[6] && p[5] && p[4])
					|| (p[6] && p[5] && p[4])
			}

			fn is_left(&self) -> bool {
				let p = self.palette;

				false
					|| (p[0] && p[7] && !p[6])
					|| (p[0] && !p[7] && p[6])
					|| (!p[0] && p[7] && !p[6])
					|| (!p[0] && p[7] && p[6])
					|| (p[0] && p[7] && p[6])
			}

			fn is_corner0(&self) -> bool {
				self.palette[0]
			}

			fn is_corner1(&self) -> bool {
				self.palette[2]
			}

			fn is_corner2(&self) -> bool {
				self.palette[4]
			}

			fn is_corner3(&self) -> bool {
				self.palette[6]
			}
		}

		#[derive(Debug)]
		enum TileLightState {
			None,
			Up,
			Right,
			Down,
			Left,
			UpLeftSmall,
			UpRightSmall,
			DownLeftSmall,
			DownRightSmall,
			UpLeftBig,
			UpRightBig,
			DownLeftBig,
			DownRightBig,
			Full,
		}

		#[derive(Debug)]
		struct TileLightTracing {
			tile_id: usize,
			rect: Rect,
			hits: usize,
			segment_hits: [usize; SEGMENT_COUNT],
		}

		impl TileLightTracing {
			fn new(tile_id: usize, position: Point2, width: f32, height: f32) -> Self {
				let rect = Rect::new(
					position.x - width / 2.0,
					position.y - height / 2.0,
					width,
					height,
				);

				Self {
					tile_id,
					rect,
					hits: 0,
					segment_hits: [0; SEGMENT_COUNT],
				}
			}

			fn find_intersection_mut(tiles: &mut Vec<Self>, point: Point2) -> Option<&mut Self> {
				for tile in tiles.iter_mut() {
					if tile.rect.contains(point) {
						return Some(tile);
					}
				}

				None
			}

			fn register_hit(&mut self, point: Point2) {
				self.hits += 1;

				let w = self.rect.w / 3.0;
				let h = self.rect.h / 3.0;

				for (segment_id, hit_count) in self.segment_hits.iter_mut().enumerate() {
					let (x, y) = match segment_id {
						0 => (self.rect.x + w * 0.0, self.rect.y + h * 0.0),
						1 => (self.rect.x + w * 1.0, self.rect.y + h * 0.0),
						2 => (self.rect.x + w * 2.0, self.rect.y + h * 0.0),
						3 => (self.rect.x + w * 2.0, self.rect.y + h * 1.0),
						4 => (self.rect.x + w * 2.0, self.rect.y + h * 2.0),
						5 => (self.rect.x + w * 1.0, self.rect.y + h * 2.0),
						6 => (self.rect.x + w * 0.0, self.rect.y + h * 2.0),
						7 => (self.rect.x + w * 0.0, self.rect.y + h * 1.0),
						_ => panic!("Invalid segment_id!"),
					};

					let segment = Rect::new(x, y, w, h);
					if segment.contains(point) {
						*hit_count += 1;
						break;
					}
				}
			}

			fn set_origin(tiles: &mut Vec<Self>, origin: Point2) {
				for tile in tiles.iter_mut() {
					tile.rect.x -= origin.x;
					tile.rect.y -= origin.y;
				}
			}

			fn get_light_state(&self) -> TileLightState {
				use TileLightState::*;

				if self.hits == 0 {
					return None;
				}

				let palette = SegmentPalette::new(&self.segment_hits);

				let up = palette.is_up();
				let right = palette.is_right();
				let down = palette.is_down();
				let left = palette.is_left();


				if up && !right && !down && !left {
					return Up;
				}
				if !up && right && !down && !left {
					return Right;
				}
				if !up && !right && down && !left {
					return Down;
				}
				if !up && !right && !down && left {
					return Left;
				}

				if up && right && !down && !left {
					return UpRightBig;
				}
				if !up && right && down && !left {
					return DownRightBig;
				}
				if !up && !right && down && left {
					return DownLeftBig;
				}
				if up && !right && !down && left {
					return UpLeftBig;
				}

				if up && !right && down && !left {
					return Full;
				}
				if !up && right && !down && left {
					return Full;
				}

				let c0 = palette.is_corner0();
				let c1 = palette.is_corner1();
				let c2 = palette.is_corner2();
				let c3 = palette.is_corner3();

				if c0 && !c1 && !c2 && !c3 {
					return UpLeftSmall;
				}
				if !c0 && c1 && !c2 && !c3 {
					return UpRightSmall;
				}
				if !c0 && !c1 && c2 && !c3 {
					return DownRightSmall;
				}
				if !c0 && !c1 && !c2 && c3 {
					return DownLeftSmall;
				}

				if c0 && !c1 && c2 && !c3 {
					return Full;
				}
				if !c0 && c1 && !c2 && c3 {
					return Full;
				}

				None
			}

			fn draw(&self, context: &mut ggez::Context, tiles: &resources::TilePack) -> ggez::GameResult<()> {
				let state = self.get_light_state();

				match state {
					TileLightState::None => {},
					TileLightState::Up => {
						graphics::draw(
							context,
							&tiles.tile_up[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::Right => {
						graphics::draw(
							context,
							&tiles.tile_up[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.rotation(90.0 * PI / 180.0)
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::Down => {
						graphics::draw(
							context,
							&tiles.tile_down[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::Left => {
						graphics::draw(
							context,
							&tiles.tile_down[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.rotation(90.0 * PI / 180.0)
								.offset(Point2::new(0.5, 0.5))
								// .color(graphics::Color::new(0.0, 0.0, 0.0, 1.0))
						)?;
					},
					TileLightState::UpLeftSmall => {
						graphics::draw(
							context,
							&tiles.corner_s[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.rotation(90.0 * PI / 180.0)
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::UpRightSmall => {
						graphics::draw(
							context,
							&tiles.corner_s[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.rotation(180.0 * PI / 180.0)
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::DownLeftSmall => {
						graphics::draw(
							context,
							&tiles.corner_s[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::DownRightSmall => {
						graphics::draw(
							context,
							&tiles.corner_s[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.rotation(270.0 * PI / 180.0)
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::UpLeftBig => {
						graphics::draw(
							context,
							&tiles.corner_b[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.rotation(270.0 * PI / 180.0)
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::UpRightBig => {
						graphics::draw(
							context,
							&tiles.corner_b[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::DownLeftBig => {
						graphics::draw(
							context,
							&tiles.corner_b[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.rotation(180.0 * PI / 180.0)
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::DownRightBig => {
						graphics::draw(
							context,
							&tiles.corner_b[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.rotation(90.0 * PI / 180.0)
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
					TileLightState::Full => {
						graphics::draw(
							context,
							&tiles.tile_up[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.offset(Point2::new(0.5, 0.5))
						)?;

						graphics::draw(
							context,
							&tiles.tile_down[0].borrow().0,
							graphics::DrawParam::default()
								.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
								.offset(Point2::new(0.5, 0.5))
						)?;
					},
				};

				Ok(())
			}
		}

		//
		//
		//

		// select tiles that are in player's radius
		let mut target_tiles = {
			let offset = self.get_level_offset(world);
			let level = &self.level.borrow();

			let mut tiles = Vec::new();

			for i in 0..level.width {
				for j in 0..level.height {
					if level.get(i, j) == resources::Wall::N {
						continue;
					}

					let tile_id = j * level.width + i;

					let tile_position = Point2::new(
						offset.x + i as f32 * WALL_SIZE + WALL_SIZE / 2.0,
						offset.y + j as f32 * WALL_SIZE + WALL_SIZE / 2.0,
					);

					let distance = Vector2::new(
							tile_position.x - self.player_coords.x,
							tile_position.y - self.player_coords.y,
						)
						.length();

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
				tile.draw(context, &self.tiles)?;
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

	fn get_tile_rect(&self, tile_id: usize) -> Rect {
		debug_assert!(tile_id < self.level.borrow().walls.len());

		let level = &self.level.borrow();

		let x = tile_id % level.width;
		let y = tile_id / level.width;

		Rect::new(x as f32 * WALL_SIZE, y as f32 * WALL_SIZE, WALL_SIZE, WALL_SIZE)
	}

	fn get_neighbor_id(&self, tile_id: usize, neighbor_n: usize) -> Option<usize> {
		debug_assert!(tile_id < self.level.borrow().walls.len());
		debug_assert!(neighbor_n < 8);

		let level = &self.level.borrow();

		let id = tile_id as isize;
		let width = level.width as isize;

		let possible_id = match neighbor_n {
			0 => id - width,
			1 => id - width + 1,
			2 => id + 1,
			3 => id + width + 1,
			4 => id + width,
			5 => id + width - 1,
			6 => id - 1,
			7 => id - width - 1,
			_ => panic!("Invalid neighbor_n: {}!", neighbor_n),
		};

		if possible_id < 0 || possible_id >= level.walls.len() as isize {
			None
		}
		else {
			Some(possible_id as usize)
		}
	}

	fn move_player(&mut self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		let direction = self.player_direction;

		// let dt = timer::duration_to_f64(timer::delta(context)) as f32;

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

			let movement_v = Vector2::new(direction.x * self.player_speed, direction.y * self.player_speed);

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
				if wall != resources::Wall::N {
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

fn center(rect: &Rect) -> Point2 {
	Point2::new(
		rect.x + rect.w,
		rect.y + rect.h,
	)
}

impl scene::Scene<World, input::Event> for LabyrinthScene {
	fn update(&mut self, world: &mut World, context: &mut ggez::Context) -> scenes::Switch {
		self.dispatcher.dispatch(&mut world.specs_world);

		self.move_player(world, context)
			.expect("Failed to move player...");

		if self.quit {
			scene::SceneSwitch::Pop
		}
		else {
			scene::SceneSwitch::None
		}
	}

	fn draw(&mut self, world: &mut World, context: &mut ggez::Context) -> ggez::GameResult<()> {
		// self.draw_level(world, context)?;
		self.draw_light(world, context);
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

//
//
//

struct TileTracing {
	//
}
