use crate::types::Error;
use crate::world::World;
use ggez::{self, graphics};
use log::{debug, info};
use serde::{Deserialize};
use std::path;
use warmy;

fn warmy_to_ggez_path(path: &path::Path, root: &path::Path) -> path::PathBuf {
    let stripped_path = path
        .strip_prefix(root)
        .expect("warmy path is outside of the warmy store?  Should never happen.");
    path::Path::new("/").join(stripped_path)
}

/// Again, because `warmy` assumes direct filesystem dirs
/// and ggez assumes all its resources live in a specific
/// (relative) location, we make our own key type here which
/// doesn't get `warmy`'s root path attached to it like its
/// `SimpleKey` types do.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceKey {
	Path(path::PathBuf),
}

impl From<&path::Path> for ResourceKey {
	fn from(p: &path::Path) -> Self {
		ResourceKey::Path(p.to_owned())
	}
}

impl ResourceKey {
	pub fn from_path<P>(p: P) -> Self
	where
		P: AsRef<path::Path>,
	{
		ResourceKey::Path(p.as_ref().to_owned())
	}
}

impl warmy::key::Key for ResourceKey {
	fn prepare_key(self, _root: &path::Path) -> Self {
		self
	}
}

/// Store and Storage are different things in `warmy`; the `Store`
/// is what actually stores things, and the `Storage` is I think
/// a handle to it.
pub type Store = warmy::Store<ggez::Context, ResourceKey>;
type Storage = warmy::Storage<ggez::Context, ResourceKey>;
pub type Loaded<T> = warmy::Loaded<T, ResourceKey>;

/// A wrapper for a ggez Image, so we can implement warmy's `Load` trait on it.
#[derive(Debug, Clone)]
pub struct Image(pub graphics::Image);

/// And, here actually tell Warmy how to load things.
impl warmy::Load<ggez::Context, ResourceKey> for Image {
	type Error = Error;
	fn load(
		key: ResourceKey,
		_storage: &mut Storage,
		context: &mut ggez::Context,
	) -> Result<Loaded<Self>, Self::Error> {
		debug!("Loading image {:?}", key);

		match key {
			ResourceKey::Path(path) => {
				graphics::Image::new(context, path)
					.map(|x| warmy::Loaded::from(Image(x)))
					.map_err(|e| Error::GgezError(e))
			},
		}
	}
}

//
//
//

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Wall {
	N,
	S,
}

pub struct TilePack {
	pub tile_up: Vec<warmy::Res<Image>>,
	pub tile_down: Vec<warmy::Res<Image>>,
	pub corner_s: Vec<warmy::Res<Image>>,
	pub corner_b: Vec<warmy::Res<Image>>,
}

impl TilePack {
	pub fn load(world: &mut World, context: &mut ggez::Context, level_key: &str) -> Self {
		const TILE_COUNT: usize = 8;
		const CORNER_COUNT: usize = 2;

		let mut tile_up = Vec::new();
		let mut tile_down = Vec::new();

		for i in 0..TILE_COUNT {
			let tile = world.resources
				.get::<Image>(&ResourceKey::from_path(&format!("/images/walls/{}/tile-{}.0.png", level_key, i)), context)
				.unwrap();
			tile_up.push(tile);

			let tile = world.resources
				.get::<Image>(&ResourceKey::from_path(&format!("/images/walls/{}/tile-{}.1.png", level_key, i)), context)
				.unwrap();
			tile_down.push(tile);
		}

		let mut corner_b = Vec::new();
		let mut corner_s = Vec::new();

		for i in 0..CORNER_COUNT {
			let tile = world.resources
				.get::<Image>(&ResourceKey::from_path(&format!("/images/walls/{}/corner-{}.0.png", level_key, i)), context)
				.unwrap();
			corner_b.push(tile);

			let tile = world.resources
				.get::<Image>(&ResourceKey::from_path(&format!("/images/walls/{}/corner-{}.1.png", level_key, i)), context)
				.unwrap();
			corner_s.push(tile);
		}

		Self {
			tile_up,
			tile_down,
			corner_s,
			corner_b,
		}
	}
}

//
//
//

#[derive(Clone, Debug, Deserialize)]
pub struct Level {
	pub walls: Vec<Wall>,
	pub width: usize,
	pub height: usize,
	pub key: String,
}

impl Level {
	pub fn get(&self, x: usize, y: usize) -> Wall {
		self.walls[self.width * y + x].clone()
	}

	pub fn load<P: AsRef<path::Path>>(context: &mut ggez::Context, file: P) -> ggez::GameResult<Self> {
		use std::io::Read;

		let mut content = String::new();
		let mut reader = ggez::filesystem::open(context, file)?;
		let _ = reader.read_to_string(&mut content)?;

		let level: Self = toml::from_str(&content)
			.map_err(|e| ggez::error::GameError::ResourceLoadError(e.to_string()))?;

		assert!(level.walls.len() == level.width * level.height);

		Ok(level)
	}

	pub fn create_test() -> Self {
		let width = 3;
		let height = 3;
		let mut walls = Vec::new();

		walls.push(Wall::S);
		walls.push(Wall::S);
		walls.push(Wall::S);

		walls.push(Wall::S);
		walls.push(Wall::N);
		walls.push(Wall::S);

		walls.push(Wall::S);
		walls.push(Wall::S);
		walls.push(Wall::S);

		Self {
			walls,
			width,
			height,
			key: "none".to_owned(),
		}
	}
}

impl warmy::Load<ggez::Context, ResourceKey> for Level {
	type Error = Error;

	fn load(
		key: ResourceKey,
		_storage: &mut Storage,
		context: &mut ggez::Context,
	) -> Result<Loaded<Self>, Self::Error> {
		debug!("Loading level {:?}", key);

		match key {
			ResourceKey::Path(path) => {
				Level::load(context, &path)
					.map(|x| warmy::Loaded::from(x))
					.map_err(|e| Error::GgezError(e))
			},
		}
	}
}
