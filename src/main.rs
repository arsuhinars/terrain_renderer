use core::app::{App, AppSettings};

mod controllers;
mod core;
mod render;
mod utils;

fn main() -> Result<(), String> {
    let app_settings = Box::new(AppSettings::default());

    let mut app = pollster::block_on(App::new(app_settings.as_ref()))?;
    app.run()?;

    Ok(())
}
