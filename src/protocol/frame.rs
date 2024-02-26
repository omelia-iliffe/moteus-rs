use num_traits::FromPrimitive;
use crate::protocol::registers::{Data, FrameRegisters};
use crate::{Register, Mode};
use crate::protocol::Resolution;

#[derive(Debug, PartialEq, Eq)]
pub enum FrameError {
    NonSequentialRegisters,
    EmptySubFrame,
    MixedReadWrites,
}
#[derive(Debug, PartialEq, Eq)]
pub enum FrameParseError {
    SubFrameRegister,
    SubFrameLength,
    Register
}
#[derive(Debug, PartialEq)]
pub struct SubFrame {
    register: FrameRegisters,
    len: u8,
    data: Vec<(Register, Data)>,
}

impl SubFrame {
    pub fn new(register: FrameRegisters, len: u8) -> Self {
        SubFrame {
            register,
            len,
            data: Vec::new(),
        }
    }
    
    fn add(&mut self, register: Register, value: Data) -> Result<(), FrameError> {
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
    pub fn add_write<T>(&mut self, register: Register, value: T) -> Result<(), FrameError> where T: Into<Data> {
        self.add(register, value.into())
    }
    pub fn add_read(&mut self, register: Register) -> Result<(), FrameError> {
        self.add(register, Data::None)
    }
    pub fn as_bytes(&self) -> Result<Vec<u8>, FrameError> {
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
        self.data.iter().map(|(_reg, value)| value).for_each(| value| {
            buf.extend_from_slice(&value.as_bytes());
        });

        Ok(buf)
    }

    pub fn from_bytes(buf: &[u8]) -> Result<(Self, usize), FrameParseError> {

            let frame_register = FrameRegisters::from_u8(buf[0] & (0xFF - 0x03)).ok_or(FrameParseError::SubFrameRegister)?;

            let (len_offset, len) = { //get len either from bits or the next byte (increments index)
                match buf[0] & 0x03 {
                    0 => {
                        (1, buf[1]) //index = 1
                    },
                    l => (0, l)
                }
            };
            let initial_reg = buf[1 + len_offset];
            let index_step = match frame_register {
                FrameRegisters::ReplyInt8 => 1,
                FrameRegisters::ReplyInt16 => 2,
                FrameRegisters::ReplyInt32 | FrameRegisters::ReplyF32 => 4,
                _ => return Err(FrameParseError::SubFrameRegister)
            };

            let start = 2 + len_offset;
            let end = (len as usize * index_step) + 2 + len_offset;
            let data = {
                let mut data = Vec::new();
                for (reg_index, i) in (start..end).step_by(index_step).enumerate() {
                    let value = match frame_register {
                        FrameRegisters::ReplyInt8 => Data::Int8(i8::from_le_bytes(buf[i..i + index_step].try_into().unwrap())),
                        FrameRegisters::ReplyInt16 => Data::Int16(i16::from_le_bytes(buf[i..i + index_step].try_into().unwrap())),
                        FrameRegisters::ReplyInt32 => Data::Int32(i32::from_le_bytes(buf[i..i + index_step].try_into().unwrap())),
                        FrameRegisters::ReplyF32 => Data::F32(f32::from_le_bytes(buf[i..i + index_step].try_into().unwrap())),
                        _ => return Err(FrameParseError::SubFrameRegister)
                    };
                    data.push((Register::from_u8(initial_reg + reg_index as u8).ok_or(FrameParseError::SubFrameRegister)?, value));
                }
                data
            };

            Ok((Self {
                register: frame_register,
                len,
                data
            }, end))
    }
}

pub struct Frame(Vec<(Register, Data)>);

impl Frame {
    pub fn from_bytes(buf: &[u8]) -> Result<Frame, FrameParseError> {
        let mut results = Vec::new();
        let mut buf = buf;
        loop {
            let (subframe, offset) = SubFrame::from_bytes(buf)?;
            // let len = subframe.len as usize;
            results.extend(subframe.data);
            buf = &buf[offset..];
            if buf.is_empty() {
                break;
            }
        }
        Ok(Frame(results))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_u8_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::WriteInt8, 1);
        subframe.add_write(Register::Mode, Mode::Stopped as i8).unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x01, 0x00, 0x00]));
    }

    #[test]
    fn test_empty_subframe() {
        let subframe = SubFrame::new(FrameRegisters::WriteInt8, 1);
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Err(FrameError::EmptySubFrame));
    }

    #[test]
    fn test_write_u16_subframe(){
        // let mut subframes = Vec::new();
        let mut subframe: SubFrame = SubFrame::new(FrameRegisters::WriteInt16, 3);
        subframe.add_write(Register::CommandPosition, 0x0060i16).unwrap();
        subframe.add_write(Register::CommandVelocity, 0x0120i16).unwrap();
        subframe.add_write(Register::CommandFeedforwardTorque, -144i16).unwrap();

        let bytes = subframe.as_bytes().unwrap();
        assert_eq!(bytes, vec![0x07, 0x20, 0x60, 0x00, 0x20, 0x01, 0x70, 0xff]);
    }

    #[test]
    fn test_read_u8_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::ReadInt8, 3);
        subframe.add_read(Register::Voltage).unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x13, 0x0d]));
    }
    #[test]
    fn test_read_u16_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::ReadInt16, 4);
        subframe.add_read(Register::Mode).unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x14, 0x04, 0x00]));
    }
    #[test]
    fn test_read_u32_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::ReadInt32, 5);
        subframe.add_read(Register::Aux1analogIn1).unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x18, 0x05, 0x60]));
    }
    #[test]
    fn parse_u8_subframe() {
        // 2404000a005000000170ff230d181400
        let buf = vec![0x23, 0x0d, 0x18, 0x14, 0x00];
        let (subframe, _) = SubFrame::from_bytes(&buf).unwrap();

        assert_eq!(subframe.register, FrameRegisters::ReplyInt8);
        assert_eq!(subframe.len, 3);
        assert_eq!(subframe.data, vec![
            (Register::Voltage, Data::Int8(24)),
            (Register::Temperature, Data::Int8(20)),
            (Register::Fault, Data::Int8(0)),
        ]);
    }
    #[test]
    fn parse_u16_subframe() {
        // 2404000a005000000170ff230d181400
        let buf = vec![0x24, 0x04, 0x00, 0x0a, 0x00, 0x50, 0x00, 0x00, 0x01, 0x70, 0xff]; // , 0x23, 0x0d, 0x18, 0x14, 0x00];
        let (subframe, _) = SubFrame::from_bytes(&buf).unwrap();

        assert_eq!(subframe.register, FrameRegisters::ReplyInt16);
        assert_eq!(subframe.len, 4);
        assert_eq!(subframe.data, vec![
            (Register::Mode, Data::Int16(10)),
            (Register::Position, Data::Int16(80)),
            (Register::Velocity, Data::Int16(256)),
            (Register::Torque, Data::Int16(-144)),
        ]);
    }
    #[test]
    fn parse_multiple_subframes() {
        let buf = vec![0x24, 0x04, 0x00, 0x0a, 0x00, 0x50, 0x00, 0x00, 0x01, 0x70, 0xff, 0x23, 0x0d, 0x18, 0x14, 0x00];
        let frame = Frame::from_bytes(&buf).unwrap();

        assert_eq!(frame.0, vec![
            (Register::Mode, Data::Int16(10)),
            (Register::Position, Data::Int16(80)),
            (Register::Velocity, Data::Int16(256)),
            (Register::Torque, Data::Int16(-144)),
            (Register::Voltage, Data::Int8(24)),
            (Register::Temperature, Data::Int8(20)),
            (Register::Fault, Data::Int8(0)),
        ]);
    }
}