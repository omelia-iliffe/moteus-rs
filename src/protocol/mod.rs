mod registers;
mod frame;

pub use registers::{Register, Mode};
pub use frame::{FrameError, SubFrame};

pub enum Resolution {
    Int8,
    Int16,
    Int32,
    Float,
}