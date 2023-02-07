use std::any::Any;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use log::error;
use wgpu::{Device, Queue, SurfaceConfiguration};

pub use super::*;

pub trait Ui {
    fn draw(&mut self, ctx: &egui::Context) { }
}

impl Ui for () { }

pub struct EguiContext<U: Ui> {
    data: U,
    window: Window,
    surface: wgpu::Surface,
    rpass: RenderPass,
    inner: egui::Context,
    state: egui_winit::State,
    config: SurfaceConfiguration,
    descriptor: ScreenDescriptor, // TODO update if window size or scale changes
    device: Device,
    queue: Queue
}

impl<U: 'static + Ui> EguiContext<U> {
    pub fn new(instance: &Instance, window: Window, evt: &EventLoop<()>, data: U) -> Self {
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
        let state = egui_winit::State::new(evt);

        Self {
            data,
            inner,
            state,
            window,
            surface,
            config,
            rpass,
            device,
            queue,
            descriptor
        }
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

}
