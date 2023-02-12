use std::any::Any;
use super::*;
use shared::Render;
use winit::{event_loop::EventLoopProxy};

pub struct RawContext<E: 'static, Data: 'static + Render<E>> {
    inner: Data,
    proxy: EventLoopProxy<E>,
    window: Window
}

impl<E: 'static, Data: 'static + Render<E>> RawContext<E, Data> {
    pub fn builder(mut data: Data) -> Box<dyn FnOnce(&Instance, Window, &EventLoop<E>) -> Self> {
        Box::new(move |instance, window, event| {
            data.init(&window);
            Self { inner: data, proxy: event.create_proxy(), window }
        })
    }
}

impl<E: 'static, Data: 'static + Render<E>> Context for RawContext<E, Data> {
    type Event = E;

    fn inner(&mut self) -> &mut Window {
        &mut self.window
    }

    fn redraw(&mut self) -> Flow {
        self.inner.render(); CONTINUE
    }

    fn resize(&mut self, physical: PhysicalSize<u32>) {
        self.inner.resize(physical.width, physical.height);
    }

    fn data(&mut self) -> Box<&mut dyn Any> {
        Box::new(&mut self.inner)
    }

    fn handle(&mut self, event: &Event<Self::Event>) {
        self.inner.handle(event);
    }
}
