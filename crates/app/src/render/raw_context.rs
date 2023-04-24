use std::any::Any;

use super::*;

pub struct RawContext<Data: 'static + Render> {
    inner: Data,
    window: Window,
}

impl<Data: 'static + Render + Default> RawContext<Data> {
    pub fn builder(ctx: &mut Emulator) -> Box<dyn FnOnce(&Instance, Window, Proxy) -> Box<dyn Context<Emulator>> + '_> {
        Box::new(move |_instance, window, _proxy| {
            let mut data = Data::default();
            data.init(&window, ctx);
            Box::new(Self { inner: data, window })
        })
    }
}

impl<Data: 'static + Render> Context<Emulator> for RawContext<Data> {
    fn inner(&mut self) -> &mut Window {
        &mut self.window
    }

    fn redraw(&mut self, emu: &mut Emulator) {
        self.inner.render(emu);
    }

    fn request_redraw(&mut self, _emu: &mut Emulator) {
        self.window.request_redraw();
    }

    fn resize(&mut self, physical: PhysicalSize<u32>, emu: &mut Emulator) {
        self.inner.resize(physical.width, physical.height, emu);
    }

    fn data(&mut self) -> Box<&mut dyn Any> {
        Box::new(&mut self.inner)
    }

    fn handle(&mut self, event: &Event, emu: &mut Emulator) {
        self.inner.handle(event, &self.window, emu);
    }
}
