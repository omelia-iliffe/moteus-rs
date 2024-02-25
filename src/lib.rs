mod bus;
mod protocol;

pub enum Resolution {
    Int8,
    Int16,
    Int32,
    Float,
}

pub use protocol::{Register, Mode};