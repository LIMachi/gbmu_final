use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow};
use winit::window::{WindowId};
use super::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Handle {
    Main, // debugger / library
    Debug,
    Game,
    Keybindings
}

pub struct Windows<E> {
    instance: Instance,
    handles: HashMap<Handle, WindowId>,
    windows: HashMap<WindowId, Box<dyn Context<Event = E>>>
}

impl<E> Windows<E> {
    pub fn new() -> Self {
        Self {
            instance: Instance::new(wgpu::Backends::PRIMARY),
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

    pub fn handle_events(&mut self, event: &Event<'_, E>, flow: &mut ControlFlow) {
        for (_, mut win) in &mut self.windows {
            win.handle(event);
        }
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id }  => {
                flow.set_exit(); },
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
        for mut window in self.windows.values_mut() {
            window.inner().request_redraw();
        }
    }

    pub fn create<C: 'static + Sized + Context<Event = E>, F: 'static + FnOnce(&Instance, Window, &EventLoop<E>) -> C>
    (&mut self, event_loop: &EventLoop<E>, window: Window, handle: Handle, mut builder: F) {
        let id = window.id();
        let ctx = Box::new(builder(&self.instance, window, event_loop));
        self.handles.insert(handle, id);
        self.windows.insert(id, ctx);
    }
}
