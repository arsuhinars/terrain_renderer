use std::{sync::Arc, thread, time::Duration};

use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use crate::render::{
    render_manager::{RenderManager, RenderSettings},
    renderer::Renderer,
    skybox_renderer::{SkyboxRenderer, SkyboxRendererSettings},
};

use super::{
    input_manager::{InputManager, InputSettings},
    time_manager::TimeManager,
};

#[derive(Clone)]
pub struct AppSettings<'a> {
    initial_size: Size,
    title: String,
    resizable: bool,
    target_frame_rate: u32,
    input_settings: InputSettings<'a>,
    render_settings: RenderSettings,
    skybox_renderer_settings: SkyboxRendererSettings,
}

impl<'a> Default for AppSettings<'a> {
    fn default() -> Self {
        Self {
            initial_size: Size::Physical(PhysicalSize::new(800, 600)),
            title: "App".into(),
            resizable: true,
            target_frame_rate: 60,
            input_settings: Default::default(),
            render_settings: Default::default(),
            skybox_renderer_settings: Default::default(),
        }
    }
}

pub struct App<'a> {
    settings: AppSettings<'a>,
    event_loop: Option<EventLoop<()>>,
    _window: Arc<Window>,
    time_manager: TimeManager,
    input_manager: InputManager<'a>,
    render_manager: RenderManager<'a>,
}

impl<'a> App<'a> {
    pub async fn new(settings: &AppSettings<'a>) -> Result<App<'a>, String> {
        let event_loop = EventLoop::new().map_err(|err| err.to_string())?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let window = Arc::new(
            WindowBuilder::new()
                .with_inner_size(settings.initial_size)
                .with_title(settings.title.clone())
                .with_resizable(settings.resizable)
                .build(&event_loop)
                .map_err(|err| err.to_string())?,
        );

        let mut render_manager =
            RenderManager::new(&settings.render_settings, window.clone()).await?;

        let renderers: Vec<Box<dyn Renderer>> = vec![Box::new(SkyboxRenderer::new(
            &settings.skybox_renderer_settings,
            &render_manager,
        ))];

        render_manager.set_renderers(renderers);

        Ok(App {
            settings: settings.clone(),
            event_loop: Some(event_loop),
            _window: window,
            time_manager: TimeManager::new(),
            input_manager: InputManager::new(&settings.input_settings),
            render_manager,
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        let event_loop = self.event_loop.take().unwrap();

        event_loop
            .run(move |event, elwt| {
                self.handle_event(event, elwt);
                self.update();
            })
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    fn handle_event(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => elwt.exit(),
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => self.render_manager.handle_resize(size),
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => self.input_manager.handle_keyboard_input(event),
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => self.input_manager.handle_cursor_movement(position),
            _ => (),
        }
    }

    fn update(&mut self) {
        self.time_manager.update();
        self.render_manager
            .render()
            .expect("Error occured while rendering");
        self.input_manager.late_update();

        let delay =
            (1.0 / (self.settings.target_frame_rate as f32) - self.time_manager.delta()).max(0.0);

        thread::sleep(Duration::from_secs_f32(delay));
    }
}