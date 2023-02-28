use std::any::Any;
use super::*;

pub struct RawContext<Data: 'static + Render> {
    inner: Data,
    proxy: Proxy,
    window: Window
}

impl<Data: 'static + Render> RawContext<Data> {
    pub fn builder(mut data: Data) -> Box<dyn FnOnce(&Instance, Window, Proxy) -> Box<dyn Context>> {
        Box::new(move |instance, window, proxy| {
            data.init(&window);
            Box::new(Self { inner: data, proxy, window })
        })
    }
}

impl<Data: 'static + Render> Context for RawContext<Data> {
    fn inner(&mut self) -> &mut Window {
        &mut self.window
    }

    fn redraw(&mut self) -> Flow {
        self.inner.render(); CONTINUE
    }

    fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    fn resize(&mut self, physical: PhysicalSize<u32>) {
        self.inner.resize(physical.width, physical.height);
    }

    fn data(&mut self) -> Box<&mut dyn Any> {
        Box::new(&mut self.inner)
    }

    fn handle(&mut self, event: &Event) {
        self.inner.handle(event, &self.window);
    }
}
