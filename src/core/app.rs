use std::{sync::Arc, time::Instant};

use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use crate::{
    controllers::camera_controller::{CameraController, CameraSettings},
    render::{
        mesh_renderer::MeshRenderer,
        render_manager::{RenderManager, RenderSettings},
        renderer::Renderer,
        skybox_renderer::{SkyboxRenderer, SkyboxRendererSettings},
        water_renderer::{WaterRenderer, WaterRendererSettings},
    },
    utils::terrain_generator::generate_terrain_mesh,
};

use super::{
    input_manager::{InputManager, InputSettings},
    time_manager::TimeManager,
};

#[derive(Clone)]
pub struct AppSettings {
    initial_size: Size,
    title: String,
    resizable: bool,
    target_frame_rate: u32,
    input_settings: InputSettings,
    render_settings: RenderSettings,
    camera_settings: CameraSettings,
    skybox_renderer_settings: SkyboxRendererSettings,
    water_renderer_settings: WaterRendererSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            initial_size: Size::Physical(PhysicalSize::new(800, 600)),
            title: "App".into(),
            resizable: true,
            target_frame_rate: 30,
            input_settings: Default::default(),
            render_settings: Default::default(),
            camera_settings: Default::default(),
            skybox_renderer_settings: Default::default(),
            water_renderer_settings: Default::default(),
        }
    }
}

pub struct App<'a> {
    event_loop: Option<EventLoop<()>>,
    _window: Arc<Window>,
    min_render_time: f32,
    last_render_time: Instant,
    time_manager: TimeManager,
    input_manager: InputManager,
    render_manager: RenderManager<'a>,
    camera_controller: CameraController,
}

impl<'a> App<'a> {
    pub async fn new(settings: &AppSettings) -> Result<App<'a>, String> {
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

        render_manager.add_renderer(Box::new(SkyboxRenderer::new(
            &settings.skybox_renderer_settings,
            &render_manager,
        )));
        render_manager.add_renderer(Box::new(MeshRenderer::new(
            generate_terrain_mesh(render_manager.device(), &Default::default()),
            &render_manager,
        )));
        render_manager.add_renderer(Box::new(WaterRenderer::new(
            &settings.water_renderer_settings,
            &render_manager,
        )));

        Ok(App {
            event_loop: Some(event_loop),
            _window: window,
            min_render_time: (1.0 / (settings.target_frame_rate as f32)),
            last_render_time: Instant::now(),
            time_manager: TimeManager::new(),
            input_manager: InputManager::new(&settings.input_settings),
            render_manager,
            camera_controller: CameraController::new(&settings.camera_settings),
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
            Event::WindowEvent {
                event: WindowEvent::CursorEntered { .. },
                ..
            } => self.input_manager.handle_cursor_enter(),
            _ => (),
        }
    }

    fn update(&mut self) {
        let instant = Instant::now();
        let t = instant.duration_since(self.last_render_time).as_secs_f32();

        if t > self.min_render_time {
            self.last_render_time = instant;
            self.time_manager.update();
            self.camera_controller.update(
                &self.time_manager,
                &self.input_manager,
                &mut self.render_manager,
            );
            self.render_manager
                .render(&self.time_manager)
                .expect("Error occured while rendering");

            self.input_manager.late_update();
        }
    }
}
