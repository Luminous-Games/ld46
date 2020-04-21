use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub mod key_codes;

const KEY_CODE_MAX: usize = 300;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum KeyCodeState {
    None = 0,
    Down = 1,
    Held = 2,
    Up = 3,
}

/// The type managing the state of the keys. Use the `key_code` module for a list
/// of KeyCodes to query for.
pub struct KeyManager {
    keys: Rc<RefCell<[KeyCodeState; KEY_CODE_MAX]>>,
}

impl KeyManager {
    pub(super) fn new() -> KeyManager {
        let window = web_sys::window().expect("global window does not exists");
        let keys = Rc::new(RefCell::new([KeyCodeState::None; KEY_CODE_MAX]));

        // REGISTER CALLBACKS

        let keys_blur = keys.clone();
        let onblur = Closure::wrap(Box::new(move |_e: web_sys::FocusEvent| {
            for key_code in 0..KEY_CODE_MAX {
                keys_blur.borrow_mut()[key_code] = KeyCodeState::Up;
            }
        }) as Box<dyn FnMut(web_sys::FocusEvent)>);
        window.set_onblur(Some(onblur.as_ref().unchecked_ref()));
        onblur.forget();

        let keys_keyup = keys.clone();
        let onkeyup = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            if (e.key_code() as usize) < key_codes::F1 || e.key_code() as usize > key_codes::F12 {
                e.prevent_default();
            }
            if e.key_code() < KEY_CODE_MAX.try_into().unwrap() {
                keys_keyup.borrow_mut()[e.key_code() as usize] = KeyCodeState::Up;
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        window.set_onkeyup(Some(onkeyup.as_ref().unchecked_ref()));
        onkeyup.forget();

        let keys_keydown = keys.clone();
        let onkeydown = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            if (e.key_code() as usize) < key_codes::F1 || e.key_code() as usize > key_codes::F12 {
                e.prevent_default();
            }
            if e.key_code() < KEY_CODE_MAX.try_into().unwrap() {
                keys_keydown.borrow_mut()[e.key_code() as usize] = KeyCodeState::Down;
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        window.set_onkeydown(Some(onkeydown.as_ref().unchecked_ref()));
        onkeydown.forget();

        KeyManager { keys }
    }

    /// Transition key states as we only get KeyCodeState::Up && KeyCodeState::Down
    /// states set from the listener, it's up to us to transition them to KeyCodeState::None
    /// && KeyCodeState::Held at the end of the frame.
    pub(super) fn post_tick_update_key_states(&mut self) {
        for index in 0..KEY_CODE_MAX {
            let value = self.keys.borrow()[index];
            match value {
                KeyCodeState::Up => self.keys.borrow_mut()[index] = KeyCodeState::None,
                KeyCodeState::Down => self.keys.borrow_mut()[index] = KeyCodeState::Held,
                _ => (),
            }
        }
    }

    /// Returns true if key was just pressed.
    ///
    /// A list of KeyCodes can be found in the `key_code` module,
    /// but any KeyCode reported by Javascript is supported.
    pub fn key_down(&self, key_code: usize) -> bool {
        self.key_state(key_code) == KeyCodeState::Down
    }

    /// Returns true every frame the key is pressed. This includes
    /// the frame where the key was just pressed as well.
    ///
    /// A list of KeyCodes can be found in the `key_code` module,
    /// but any KeyCode reported by Javascript is supported.
    pub fn key_pressed(&self, key_code: usize) -> bool {
        match self.key_state(key_code) {
            KeyCodeState::Down | KeyCodeState::Held => true,
            _ => false,
        }
    }

    /// Returns true if key was just released.
    ///
    /// A list of KeyCodes can be found in the `key_code` module,
    /// but any KeyCode reported by Javascript is supported.
    pub fn key_up(&self, key_code: usize) -> bool {
        self.key_state(key_code) == KeyCodeState::Up
    }

    /// Returns the KeyCodeState for a given KeyCode.
    fn key_state(&self, key_code: usize) -> KeyCodeState {
        self.keys.borrow()[key_code]
    }
}
