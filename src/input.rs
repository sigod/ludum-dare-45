//! Typedefs for input shortcuts.
use ggez::event::*;
use ggez_goodies::input;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Button {
	Next,
	Quit,

	Up,
	Down,
	Left,
	Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Axis {
	Vert,
	Horz,
}

pub type Binding = input::InputBinding<Axis, Button>;
pub type Event = input::InputEffect<Axis, Button>;
pub type State = input::InputState<Axis, Button>;

/// Create the default keybindings for our input state.
pub fn create_input_binding() -> input::InputBinding<Axis, Button> {
	input::InputBinding::new()
		.bind_key_to_button(KeyCode::Up, Button::Up)
		.bind_key_to_button(KeyCode::Down, Button::Down)
		.bind_key_to_button(KeyCode::Left, Button::Left)
		.bind_key_to_button(KeyCode::Right, Button::Right)

		.bind_key_to_button(KeyCode::W, Button::Up)
		.bind_key_to_button(KeyCode::S, Button::Down)
		.bind_key_to_button(KeyCode::A, Button::Left)
		.bind_key_to_button(KeyCode::D, Button::Right)

		.bind_key_to_button(KeyCode::Space, Button::Next)
		.bind_key_to_button(KeyCode::Escape, Button::Quit)
}
