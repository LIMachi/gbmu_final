use std::collections::HashMap;
use std::ops::Deref;
use egui::panel::Side;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use log::error;
use wgpu::{CommandEncoder, Device, Instance, InstanceDescriptor, Queue};
use winit::{
    window::{Window, WindowBuilder},
    event_loop::{EventLoop, ControlFlow, EventLoopWindowTarget},
    event::{Event, WindowEvent}
};
use winit::window::WindowId;

mod app;

pub type Flow = std::ops::ControlFlow<()>;
pub const CONTINUE: Flow = Flow::Continue(());
pub const BREAK: Flow = Flow::Break(());

pub trait Context {
    fn redraw(&mut self) -> Flow;
}

pub struct PixelContext {
    window: Window
}

pub trait Ui {
    fn draw(&mut self, ctx: &egui::Context) { }
}

impl Ui for () { }

pub struct Debugger { }

impl Ui for Debugger {
    fn draw(&mut self, ctx: &egui::Context) {
        egui::SidePanel::new(Side::Left, "sidebar").show(ctx, |_|{ });
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.heading("DEBUGGING TIME");
            });
    }
}

pub struct EguiContext<U: Ui> {
    data: U,
    window: Window,
    surface: wgpu::Surface,
    rpass: RenderPass,
    inner: egui::Context,
    state: egui_winit::State,
    descriptor: ScreenDescriptor, // TODO update if window size or scale changes
    device: Device,
    queue: Queue
}

impl<U: 'static + Ui> EguiContext<U> {
    pub fn new(instance: &wgpu::Instance, window: Window, evt: &EventLoop<()>, data: U) -> Self {
        let surface = unsafe { instance.create_surface(&window).expect("can't create surface") };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        })).expect("no suitable adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                label: None
            },
            None)).expect("no matching device");
        let size = window.inner_size();
        let config = surface.get_default_config(&adapter, size.width as u32, size.height as u32).expect("unsupported");
        surface.configure(&device, &config);
        let rpass = RenderPass::new(&device, config.format, 1);
        let descriptor = ScreenDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor() as f32
        };
        let inner = egui::Context::default();
        let state = egui_winit::State::new(evt.deref());

        Self {
            data,
            inner,
            state,
            window,
            surface,
            rpass,
            device,
            queue,
            descriptor
        }
    }
}

impl<U: Ui> Context for EguiContext<U> {
    fn redraw(&mut self) -> Flow {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => return CONTINUE,
            Err(e) => {
                error!("Dropped frame: {e:?}");
                return CONTINUE;
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let raw = self.state.take_egui_input(&self.window);
        let out = self.inner.run(raw, |ctx| { self.data.draw(ctx); });
        self.state.handle_platform_output(&self.window, &self.inner, out.platform_output);
        let jobs = self.inner.tessellate(out.shapes);
        let delta = out.textures_delta;
        self.rpass.add_textures(&self.device, &self.queue, &delta);
        self.rpass.update_buffers(
            &self.device,
            &self.queue,
            &jobs,
            &self.descriptor
        );
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });
        self.rpass.execute(&mut encoder, &view, &jobs, &self.descriptor, Some(wgpu::Color::BLACK)).unwrap();
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        self.rpass.remove_textures(delta).expect("gpu crashed. oh well.");
        CONTINUE
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Handle {
    Main, // debugger / library
    Game,
    Keybindings,
    SpriteSheet
}
#[derive(Default)]
pub struct Windows {
    handles: HashMap<Handle, WindowId>,
    windows: HashMap<WindowId, Box<dyn Context>>
}

pub struct App {
    instance: Instance,
    event_loop: Option<EventLoop<()>>,
    handles: HashMap<Handle, WindowId>,
    windows: HashMap<WindowId, Box<dyn Context>>,
    inner: Windows
}

impl App {
    pub fn new() -> Self {
        Self {
            instance: Instance::new(InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() }),
            event_loop: Some(EventLoop::new()),
            handles: Default::default(),
            windows: Default::default(),
            inner: Default::default()
        }
    }

    pub fn create<C: 'static + Sized + Context, F: 'static + Fn(&Instance, Window, &EventLoop<()>) -> C>(mut self, handle: Handle, builder: F) -> Self {
        let win = Window::new(self.event_loop.as_ref().unwrap()).unwrap();
        let id = win.id();
        let ctx = Box::new(builder(&self.instance, win, self.event_loop.as_ref().unwrap()));
        self.handles.insert(handle, id);
        self.windows.insert(id, ctx);
        self
    }

    pub fn run<F: 'static + FnMut()>(mut self, mut handler: F) -> ! {
        let event = self.event_loop.take().expect("yeah no");
        event.run(move |event: Event<'_, ()>, target: &EventLoopWindowTarget<()>, flow: &mut ControlFlow| {
            flow.set_wait();
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, window_id }  => {
                    flow.set_exit(); },
                Event::WindowEvent { event: WindowEvent::Resized(sz), window_id } => {

                },
                Event::RedrawRequested(id) => {
                    self.windows.get_mut(&id).unwrap().redraw();
                },
                Event::MainEventsCleared => { handler(); },
                _ => ()
            }
        })
    }
}

mod debugger {
    use winit::window::Window;
    use wgpu::Instance;
    use winit::event_loop::EventLoop;
    use crate::{Context, Debugger, EguiContext};

    pub fn create(instance: &Instance, window: Window, evt: &EventLoop<()>) -> impl Context {
        EguiContext::new(instance, window, evt, Debugger { })
    }
}

fn main() {
    let mut emu = app::Emu::new();
    App::new()
        .create(Handle::Main, debugger::create)
        .run(move || { emu.cycle(); });
}
