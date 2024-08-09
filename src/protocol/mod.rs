mod frame;
pub mod registers;

pub use frame::{Frame, FrameBuilder, ResponseFrame};

/// Moteus register can be read in multiple resolutions (`Int8`, `Int16`, `Int32`, `Float`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resolution {
    /// An 8-bit integer. Some registers expect a signed 8-bit integer, while others expect an unsigned 8-bit integer.
    Int8,
    /// A 16-bit integer. Some registers expect a signed 16-bit integer, while others expect an unsigned 16-bit integer.
    Int16,
    /// A 32-bit integer. Some registers expect a signed 32-bit integer, while others expect an unsigned 32-bit integer.
    Int32,
    /// A 32-bit floating point number as defined in IEEE 754-2008.
    Float,
}

impl Resolution {
    /// Returns the number of bytes for the resolution.
    pub fn size(&self) -> usize {
        match self {
            Resolution::Int8 => 1,
            Resolution::Int16 => 2,
            Resolution::Int32 => 4,
            Resolution::Float => 4,
        }
    }
}
