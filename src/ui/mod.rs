mod app;
mod screen;
//mod util;

pub use app::run;

pub type Result<T> = crossterm::Result<T>;
