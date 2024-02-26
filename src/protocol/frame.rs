use crate::protocol::registers::FrameRegisters;
use crate::{Register};

#[derive(Debug, PartialEq, Eq)]
enum FrameError {
    NonSequentialRegisters,
    EmptySubFrame,
    MixedReadWrites,
}
struct SubFrame<T> {
    register: FrameRegisters,
    len: u8,
    data: Vec<(Register, Option<T>)>,
}

impl<T> SubFrame<T>
    where T: zerocopy::AsBytes {
    fn new(register: FrameRegisters, len: u8) -> Self {
        SubFrame {
            register,
            len,
            data: Vec::new(),
        }
    }
    
    fn add(&mut self, register: Register, value: Option<T>) -> Result<(), FrameError> {
        if let Some((last_register, last_value)) = self.data.last() {
            if *last_register as u8 + 1 != register as u8 { //TODO: SHOULD NOT BE U8
                return Err(FrameError::NonSequentialRegisters);
            }
            if last_value.is_none() != value.is_none() {
                return Err(FrameError::MixedReadWrites)
            }
        }
        self.data.push((register, value));
        Ok(())
    }
    fn add_write(&mut self, register: Register, value: T) -> Result<(), FrameError> {
        self.add(register, Some(value))
    }
    fn add_read(&mut self, register: Register) -> Result<(), FrameError> {
        self.add(register, None)
    }
    fn as_bytes(&self) -> Result<Vec<u8>, FrameError> {
        let mut buf = Vec::with_capacity(64); //TODO: SWAP WITH SUB FRAME BUF TO AVOID ALLOCATING
        if self.len < 4 {
            buf.push((self.register as u8) | self.len);
        } else {
            buf.push(self.register as u8);
            buf.push(self.len);
        }
        // write registers, takes into account unneeded sequential registers
        let first_register = self.data.first().ok_or(FrameError::EmptySubFrame)?.0 as u8; //TODO: SHOULD NOT BE U8
        buf.push(first_register);
        self.data.iter().map(|(_reg, value)| value).flatten().for_each(| value| {
            buf.extend_from_slice(value.as_bytes());
        });

        Ok(buf)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_u8_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::WRITE_INT8, 1);
        subframe.add_write(Register::kMode, Mode::kStopped);
        let bytes = dbg!(subframe.as_bytes());

        assert_eq!(bytes, Ok(vec![0x01, 0x00, 0x00]));
    }

    #[test]
    fn test_empty_subframe() {
        let subframe: SubFrame<u8> = SubFrame::new(FrameRegisters::WRITE_INT8, 1);
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Err(FrameError::EmptySubFrame));
    }

    #[test]
    fn test_write_u16_subframe(){
        // let mut subframes = Vec::new();
        let mut subframe: SubFrame<u16> = SubFrame::new(FrameRegisters::WRITE_INT16, 3);
        subframe.add_write(Register::kCommandPosition, 0x0060).unwrap();
        subframe.add_write(Register::kCommandVelocity, 0x0120).unwrap();
        subframe.add_write(Register::kCommandFeedforwardTorque, 0xff50).unwrap();

        let bytes = subframe.as_bytes().unwrap();
        assert_eq!(bytes, vec![0x07, 0x20, 0x60, 0x00, 0x20, 0x01, 0x50, 0xff]);
    }

    #[test]
    fn test_read_u8_subframe() {
        let mut subframe: SubFrame<u8> = SubFrame::new(FrameRegisters::READ_INT8, 3);
        subframe.add_read(Register::kVoltage).unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x13, 0x0d]));
    }
    #[test]
    fn test_read_u16_subframe() {
        let mut subframe: SubFrame<u8> = SubFrame::new(FrameRegisters::READ_INT16, 4);
        subframe.add_read(Register::kMode).unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x14, 0x04, 0x00]));
    }
    #[test]
    fn test_read_u32_subframe() {
        let mut subframe: SubFrame<u8> = SubFrame::new(FrameRegisters::READ_INT32, 5);
        subframe.add_read(Register::kAux1AnalogIn1).unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x18, 0x05, 0x60]));
    }
}