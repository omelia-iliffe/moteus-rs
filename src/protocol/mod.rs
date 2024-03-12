pub mod registers;
mod frame;
#[macro_use]
mod register_macros;

pub use frame::{Frame, FrameBuilder, FrameError, FrameParseError, ResponseFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resolution {
    Int8,
    Int16,
    Int32,
    Float,
}
