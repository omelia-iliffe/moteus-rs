//! A trait for writing and reading CAN FD frames over an interface.

#[cfg(feature = "fdcanusb")]
mod fdcanusb;

pub trait Transport {
    type Error;

    type Frame;

    fn transmit(&mut self, frame: Self::Frame) -> Result<(), crate::Error<Self::Error>>;

    fn receive(&mut self) -> Result<Self::Frame, crate::Error<Self::Error>>;
}
