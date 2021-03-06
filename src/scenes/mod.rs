use crate::input;
use crate::world::World;
use ggez_goodies::scene;

pub mod labyrinth;
pub mod transition;

pub type Switch = scene::SceneSwitch<World, input::Event>;
pub type Stack = scene::SceneStack<World, input::Event>;
