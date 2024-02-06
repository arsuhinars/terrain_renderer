use core::app::App;

mod core;
mod render;
mod utils;

fn main() -> Result<(), String> {
    let mut app = pollster::block_on(App::new(&Default::default()))?;
    app.run()?;

    Ok(())
}
