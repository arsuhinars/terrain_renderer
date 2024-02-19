use core::app::{App, AppSettings};

pub mod controllers;
pub mod core;
pub mod render;
pub mod utils;

fn main() -> Result<(), String> {
    let app_settings = Box::new(AppSettings::default());

    let mut app = pollster::block_on(App::new(app_settings.as_ref()))?;
    app.run()?;

    Ok(())
}
