use crate::transport::Transport;
use crate::Error;
use fdcanusb::FdCanUSB;

impl Transport for FdCanUSB<fdcanusb::serial2::SerialPort> {
    type Error = fdcanusb::TransferError;
    type Frame = fdcanusb::CanFdFrame;

    fn transmit(&mut self, frame: Self::Frame) -> Result<(), Error<Self::Error>> {
        self.write(frame).map_err(Error::Transport)
    }

    fn receive(&mut self) -> Result<Self::Frame, Error<Self::Error>> {
        self.read()
            .map_err(fdcanusb::TransferError::Read)
            .map_err(Error::Transport)
    }
}
