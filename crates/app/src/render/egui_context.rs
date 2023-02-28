use std::any::Any;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use log::error;
use wgpu::{CompositeAlphaMode, Device, PresentMode, Queue, SurfaceConfiguration, TextureFormat, TextureUsages};
use winit::event::WindowEvent;
use shared::{Ui, egui};

pub use super::*;

pub struct EguiContext<U: Ui> {
    data: U,
    window: Window,
    surface: wgpu::Surface,
    rpass: RenderPass,
    proxy: Proxy,
    platform: Platform,
    inner: egui::Context,
    config: SurfaceConfiguration,
    descriptor: ScreenDescriptor, // TODO update if window size or scale changes
    device: Device,
    queue: Queue
}

impl<U: 'static + Ui> EguiContext<U> {
    pub fn new(instance: &Instance, window: Window, proxy: Proxy, mut data: U) -> Self {
        let surface = unsafe { instance.create_surface(&window) };
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
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::default(),
            alpha_mode: CompositeAlphaMode::Auto
        };

        surface.configure(&device, &config);
        let rpass = RenderPass::new(&device, config.format, 1);
        let descriptor = ScreenDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor() as f32
        };
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: Default::default(),
            style: Default::default()
        });
        let mut inner = platform.context();
        data.init(&mut inner);
        Self {
            data,
            inner,
            window,
            surface,
            config,
            platform,
            rpass,
            device,
            proxy,
            queue,
            descriptor
        }
    }

    pub fn builder(data: U) -> Box<dyn FnOnce(&Instance, Window, Proxy) -> Box<dyn Context>> {
        Box::new(move |instance, window, event|
            Box::new(Self::new(instance, window, event, data)))
    }
}

impl<U: 'static + Ui> Context for EguiContext<U> {
    fn inner(&mut self) -> &mut Window {
        &mut self.window
    }

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
        self.platform.begin_frame();
        self.data.draw(&mut self.inner);
        let out = self.platform.end_frame(Some(&self.window));
        let jobs = self.inner.tessellate(out.shapes);
        let delta = out.textures_delta;
        self.rpass.add_textures(&self.device, &self.queue, &delta).ok();
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

    fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    fn resize(&mut self, physical: PhysicalSize<u32>) {
        if physical.width > 0 && physical.height > 0 {
            self.config.width = physical.width;
            self.config.height = physical.height;
            self.descriptor.physical_width = physical.width;
            self.descriptor.physical_height = physical.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn data(&mut self) -> Box<&mut dyn Any> {
        Box::new(&mut self.data)
    }

    fn handle(&mut self, event: &Event) {
        match event {
            Event::WindowEvent { window_id, .. } if window_id == &self.window.id() => {
                self.platform.handle_event(event);
                self.data.handle(event);
            },
            Event::UserEvent(_) => self.data.handle(event),
            Event::WindowEvent { event: wevent, .. } => {
                match wevent {
                    WindowEvent::CursorEntered { .. } | WindowEvent::CursorLeft { .. } | WindowEvent::CursorMoved { ..} => {
                        self.platform.handle_event(event);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}
