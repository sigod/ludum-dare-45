use crate::level_configuration::{LevelConfiguration};
use crate::resources;
use crate::types::{Point2, Rect};
use ggez::graphics;
use std::f32::consts::PI;

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
pub struct TileLightTracing {
	pub tile_id: usize,
	pub rect: Rect,
	pub hits: usize,
	pub segment_hits: [usize; SEGMENT_COUNT],
}

impl TileLightTracing {
	pub fn new(tile_id: usize, position: Point2, width: f32, height: f32) -> Self {
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

	pub fn find_intersection_mut(tiles: &mut Vec<Self>, point: Point2) -> Option<&mut Self> {
		for tile in tiles.iter_mut() {
			if tile.rect.contains(point) {
				return Some(tile);
			}
		}

		None
	}

	pub fn register_hit(&mut self, point: Point2) {
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

	pub fn set_origin(tiles: &mut Vec<Self>, origin: Point2) {
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

	pub fn draw(&self, context: &mut ggez::Context, tiles: &resources::TilePack, level_configuration: &LevelConfiguration) -> ggez::GameResult<()> {
		let state = self.get_light_state();

		let side_n = level_configuration.get_side(self.tile_id);
		let corner_n = level_configuration.get_corner(self.tile_id);

		match state {
			TileLightState::None => {},
			TileLightState::Up => {
				graphics::draw(
					context,
					&tiles.tile_up[side_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::Right => {
				graphics::draw(
					context,
					&tiles.tile_up[side_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.rotation(90.0 * PI / 180.0)
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::Down => {
				graphics::draw(
					context,
					&tiles.tile_down[side_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::Left => {
				graphics::draw(
					context,
					&tiles.tile_down[side_n].borrow().0,
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
					&tiles.corner_s[corner_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.rotation(90.0 * PI / 180.0)
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::UpRightSmall => {
				graphics::draw(
					context,
					&tiles.corner_s[corner_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.rotation(180.0 * PI / 180.0)
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::DownLeftSmall => {
				graphics::draw(
					context,
					&tiles.corner_s[corner_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::DownRightSmall => {
				graphics::draw(
					context,
					&tiles.corner_s[corner_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.rotation(270.0 * PI / 180.0)
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::UpLeftBig => {
				graphics::draw(
					context,
					&tiles.corner_b[corner_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.rotation(270.0 * PI / 180.0)
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::UpRightBig => {
				graphics::draw(
					context,
					&tiles.corner_b[corner_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::DownLeftBig => {
				graphics::draw(
					context,
					&tiles.corner_b[corner_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.rotation(180.0 * PI / 180.0)
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::DownRightBig => {
				graphics::draw(
					context,
					&tiles.corner_b[corner_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.rotation(90.0 * PI / 180.0)
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
			TileLightState::Full => {
				graphics::draw(
					context,
					&tiles.tile_up[side_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.offset(Point2::new(0.5, 0.5))
				)?;

				graphics::draw(
					context,
					&tiles.tile_down[side_n].borrow().0,
					graphics::DrawParam::default()
						.dest(Point2::new(self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0))
						.offset(Point2::new(0.5, 0.5))
				)?;
			},
		};

		Ok(())
	}
}
