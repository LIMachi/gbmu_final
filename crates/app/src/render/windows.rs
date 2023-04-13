use std::any::Any;
use std::collections::HashMap;
use winit::event::{WindowEvent};
use winit::event_loop::{ControlFlow};
use winit::window::{WindowId};
use super::*;

pub struct Windows {
    instance: Instance,
    proxy: Proxy,
    handles: HashMap<Handle, WindowId>,
    windows: HashMap<WindowId, Box<dyn Context<Emulator>>>
}

impl Windows {
    pub fn new(proxy: Proxy) -> Self {
        Self {
            instance: Instance::new(wgpu::Backends::PRIMARY),
            proxy,
            handles: Default::default(),
            windows: Default::default()
        }
    }

    /**
        Retrieves inner data from the associated handle, if any.
        It will be returned as a Box<dyn Any> which can be downcasted to the inner type.
        Preferably don't use, unless you know exactly what you're getting.

        Handle::Main => Debugger
        _ => Unused
    **/
    pub fn get_mut(&mut self, handle: Handle) -> Option<Box<&mut dyn Any>> {
        self.handles
            .get_mut(&handle)
            .and_then(|x| self.windows.get_mut(x))
            .map(|x| x.data())
    }

    pub fn is_open(&self, handle: Handle) -> bool {
        self.handles.contains_key(&handle)
    }

    pub fn handle_events(&mut self, event: &Event, flow: &mut ControlFlow) {
        for (_, win) in &mut self.windows {
            win.handle(event);
        }
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id }  => {
                if let Some((&h, &_id)) = self.handles.iter().find(|(_, v)| v == &window_id) {
                    if self.windows.contains_key(window_id) && self.windows.len() == 1 || h == Handle::Main {
                        self.proxy.send_event(Events::Close).ok();
                        flow.set_exit();
                    } else {
                        self.handles.remove(&h);
                        self.windows.remove(window_id);
                    }
                }
            },
            Event::WindowEvent { event: WindowEvent::Resized(sz), window_id } => {
                self.windows.get_mut(&window_id).unwrap().resize(*sz);
            },
            Event::RedrawRequested(id) => {
                self.windows.get_mut(&id).unwrap().redraw();
            },
            _ => {}
        }
    }

    pub fn update(&mut self) {
        for window in self.windows.values_mut() {
            window.request_redraw();
        }
    }

    pub fn create<'a>(&mut self, handle: Handle, emu: &mut Emulator, event_loop: &EventLoopWindowTarget<Events>) {
        if self.handles.contains_key(&handle) {
            log::warn!("window {handle:?} already opened. Please don't do that.");
            return ;
        }
        let proxy = self.proxy.clone();
        let kind = WindowType(handle);
        let window = kind.build(event_loop);
        let id = window.id();

        let ctx = kind.builder(emu)(&self.instance, window, proxy);

        self.handles.insert(handle, id);
        self.windows.insert(id, ctx);
    }
}
