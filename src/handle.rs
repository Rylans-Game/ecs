
use std::any::{Any, type_name};

pub trait Handle: 'static {
    fn handle() -> &'static mut u16;
    fn name() -> &'static str;
}

impl<T: Any> Handle for T {
    fn handle() -> &'static mut u16 {
        static mut ID: u16 = u16::MAX;
        unsafe { &mut ID }
    }

    fn name() -> &'static str {
        type_name::<Self>()
    }
}

pub trait Resource : Handle {}
pub trait Component : Handle {}