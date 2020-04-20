use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use num::FromPrimitive;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use self::event::{MouseEvent, MouseEventType};
pub use self::state::MouseState;

pub const MOUSE_EVENT_MAX: usize = 1000;

#[repr(u32)]
#[derive(Clone, Copy, FromPrimitive)]
pub enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
}

mod event;
mod state;

/// The type tracking and managing the state of the mouse
pub struct MouseManager {
    mouse_events: Rc<RefCell<[MouseEvent; MOUSE_EVENT_MAX]>>,
    mouse_events_index: Rc<RefCell<usize>>,
    mouse_state: MouseState,
}

impl MouseManager {
    pub(super) fn new() -> MouseManager {
        let window = web_sys::window().expect("global window does not exists");
        let mouse_events = Rc::new(RefCell::new([MouseEvent::NONE; MOUSE_EVENT_MAX]));
        let mouse_events_index = Rc::new(RefCell::new(0));

        for (js_type, mouse_type) in &[
            ("mousedown", MouseEventType::Down),
            ("mouseup", MouseEventType::Up),
            ("mouseover", MouseEventType::Over),
            ("mouseout", MouseEventType::Out),
            ("mousemove", MouseEventType::Move),
        ] {
            let cloned_events = mouse_events.clone();
            let cloned_events_index = mouse_events_index.clone();
            let callback = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                cloned_events.borrow_mut()[*cloned_events_index.borrow()] = MouseEvent {
                    event_type: *mouse_type,
                    button: FromPrimitive::from_i16(e.button()).unwrap(),
                    pos_mx: e.x() as u32,
                    pos_my: e.y() as u32,
                };
                *cloned_events_index.borrow_mut() += 1;
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            window
                .add_event_listener_with_callback(js_type, callback.as_ref().unchecked_ref())
                .unwrap();
            callback.forget();
        }

        MouseManager {
            mouse_events,
            mouse_events_index,
            mouse_state: MouseState::default(),
        }
    }

    /// Updates the mouse state given the list of events
    pub(super) fn pre_tick_process_mouse_state(&mut self) {
        self.mouse_state.start_update();

        for event in self
            .mouse_events
            .borrow()
            .iter()
            .take(*self.mouse_events_index.borrow())
        {
            if event.event_type == MouseEventType::None {
                break;
            }

            self.mouse_state.update(event);
        }

        *self.mouse_events_index.borrow_mut() = 0;
    }
}

impl Deref for MouseManager {
    type Target = MouseState;

    fn deref(&self) -> &Self::Target {
        &self.mouse_state
    }
}
