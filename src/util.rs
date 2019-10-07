use crate::types::{Point2, Rect, Vector2};
use ggez;

/// A couple handy re-exports from Euclid
// pub use euclid::point2;
// pub use euclid::vec2;

/// Basic logging setup to log to the console with `fern`.
pub fn setup_logging() {
	use fern::colors::{Color, ColoredLevelConfig};
	let colors = ColoredLevelConfig::default()
		.info(Color::Green)
		.debug(Color::BrightMagenta)
		.trace(Color::BrightBlue);
	// This sets up a `fern` logger and initializes `log`.
	fern::Dispatch::new()
		// Formats logs
		.format(move |out, message, record| {
			out.finish(format_args!(
				"[{}][{:<5}][{}] {}",
				chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
				colors.color(record.level()),
				record.target(),
				message
			))
		})
		.level(log::LevelFilter::Warn)
		// Filter out unnecessary stuff
		// .level_for("gfx", log::LevelFilter::Off)
		// .level_for("walk", log::LevelFilter::Warn)
		// Set levels for stuff we care about
		.level_for("ludum_dare_45", log::LevelFilter::Info)
		.level_for("winit::platform::platform::window", log::LevelFilter::Info)
		// .level_for("threething", log::LevelFilter::Trace)
		// Hooks up console output.
		// env var for outputting to a file?
		// Haven't needed it yet!
		.chain(std::io::stdout())
		// TODO: Log into file!
		// .chain(fern::log_file("output.log").expect("Could not open log file!"))
		.apply()
		.expect("Could not init logging!");
}

pub fn get_distance(a: Point2, b: Point2) -> f32 {
		Vector2::new(a.x - b.x, a.y - b.y).length()
	}

fn check_intersection(a: f32, aw: f32, b: f32, bw: f32) -> bool {
	// info!("check_intersection: {} < {} + {} && {} < {} + {} = {}", a, b, bw, b, a, aw, a < b + bw && b < a + aw);
	a < b + bw && b < a + aw
}

pub fn check_collision(a: Rect, b: Rect) -> bool {
	if check_intersection(a.x, a.w, b.x, b.w)
		&& check_intersection(a.y, a.h, b.y, b.h)
	{
		true
	}
	else {
		false
	}
}
