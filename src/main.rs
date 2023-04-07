use std::thread;
use std::time::{Duration, Instant};

use rand::Rng;
use wgpu::{Device, Queue};
use wgpu::{Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod life;
mod render;

struct State {
    window: Window,
    surface: Surface,
    window_config: SurfaceConfiguration,
    window_size: PhysicalSize<u32>,
    device: Device,
    queue: Queue,
    renderer: render::Renderer,
    life: life::Life,
    last_frame: Instant,
}

impl State {
    async fn new(window: Window, data: Vec<u32>, params: life::Params) -> Self {
        // UNIVERSAL GPU INITIALISATION
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        println!("{:#?}", adapter.limits());
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        // SETTING UNIVERSAL WINDOW STUFF (no move)
        let window_size = window.inner_size();
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let window_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &window_config);

        // SHARED BETWEEN LIFE AND RENDERER HAS TO BE HERE
        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: params.width,
                height: params.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            format: wgpu::TextureFormat::Rgba32Float,
            view_formats: &[],
        });
        let output_texture_view =
            output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // INIT COMPUTE SHADER (move to life.rs)
        let life = life::Life::new(data, params, &device, &output_texture_view);

        // PURE RENDERER STUFF
        let renderer = render::Renderer::new(&device, output_texture_view, &window_config);

        let last_frame = Instant::now();

        Self {
            window,
            surface,
            window_config,
            window_size,
            device,
            queue,
            renderer,
            life,
            last_frame,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    async fn update(&mut self) {
        thread::sleep(Duration::from_millis(30));
        let elapsed = self.last_frame.elapsed().as_micros() as f32;
        println!("{}ms since last update", elapsed / 1000.0);
        println!("({} fps)", 1000000.0 / elapsed);
        self.last_frame = Instant::now();
        self.life.step(&self.device, &self.queue).await;
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.window_config.width = new_size.width;
            self.window_config.height = new_size.height;
            self.surface.configure(&self.device, &self.window_config);
        }
    }

    fn input(&mut self, _: &WindowEvent) -> bool {
        false
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer
            .render(&self.surface, &self.device, &self.queue)
    }
}

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut rng = rand::thread_rng();
    let data = (0..(WIDTH * HEIGHT))
        .map(|_| rng.gen_bool(0.5) as u32 * 200)
        .collect::<Vec<_>>();
    let mut state = State::new(window, data, life::Params::new(WIDTH, HEIGHT)).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                pollster::block_on(state.update());
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.window_size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    pollster::block_on(run());
}
