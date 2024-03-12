use std::collections::HashMap;

use fdcanusb::CanFdFrame;
use itertools::Itertools;
use num_traits::FromPrimitive;

use crate::protocol::registers::{FrameRegisters, RegisterDataStruct, RegisterError};
use crate::registers::{Register, RegisterAddr};
use crate::Resolution;

#[derive(Debug, PartialEq)]
pub enum FrameError {
    NonSequentialRegisters,
    EmptySubFrame,
    MixedReadWrites,
    RegisterError(RegisterError),
}

impl From<RegisterError> for FrameError {
    fn from(e: RegisterError) -> Self {
        FrameError::RegisterError(e)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FrameParseError {
    SubFrameRegister,
    SubFrameLength,
    Register,
}

#[derive(Debug, PartialEq)]
pub struct SubFrame {
    register: FrameRegisters,
    len: u8,
    data: Vec<RegisterDataStruct>,
}

impl SubFrame {
    pub fn new(register: FrameRegisters, len: u8) -> Self {
        SubFrame {
            register,
            len,
            data: Vec::new(),
        }
    }

    fn add(&mut self, register: RegisterDataStruct) -> Result<(), FrameError> {
        if let Some(prev_reg) = self.data.last() {
            if (prev_reg.address as u16) + 1 != register.address as u16 {
                //TODO: SHOULD NOT BE U8
                return Err(FrameError::NonSequentialRegisters);
            }
            if register.data.is_none() != register.data.is_none() {
                return Err(FrameError::MixedReadWrites);
            }
        }
        self.data.push(register);
        Ok(())
    }
    // pub fn add_write<T>(&mut self, register: RegisterAddr, value: T) -> Result<(), FrameError>
    //     where
    //         T: Into<Data>,
    // {
    //     self.add(register, value.into())
    // }
    // pub fn add_read(&mut self, register: RegisterAddr) -> Result<(), FrameError> {
    //     self.add(register, Data::None)
    // }
    pub(crate) fn as_bytes(&self) -> Result<Vec<u8>, FrameError> {
        let mut buf = Vec::with_capacity(64); //TODO: SWAP WITH SUB FRAME BUF TO AVOID ALLOCATING
        if self.len < 4 {
            buf.push((self.register as u8) | self.len);
        } else {
            buf.push(self.register as u8);
            buf.push(self.len);
        }
        // write registers, takes into account unneeded sequential registers
        let first_reg = self.data.first().ok_or(FrameError::EmptySubFrame)?;
        buf.extend(first_reg.address.address_as_bytes());
        if first_reg.data.is_some() {
            self.data.iter().for_each(|reg| {
                buf.extend_from_slice(reg.clone().data.unwrap().as_slice()); //TODO: remove extra clone
            });
        }

        Ok(buf)
    }

    pub(crate) fn from_bytes(buf: &[u8]) -> std::io::Result<(Option<Self>, usize)> {
        if buf.is_empty() {
            return Ok((None, 0));
        }
        let frame_register =
            FrameRegisters::from_u8(buf[0] & (0xFF - 0x03)).ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown FrameRegister {}", buf[0] & (0xFF - 0x03)),
            ))?;
        if frame_register == FrameRegisters::Nop {
            return Ok((None, 1));
        }
        let (len_offset, len) = {
            //get len either from bits or the next byte (increments index)
            match buf[0] & 0x03 {
                0 => {
                    (1, buf[1]) //index = 1
                }
                l => (0, l),
            }
        };
        let initial_reg = buf[1 + len_offset];
        let index_step = frame_register.size().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Expected a Reply FrameRegister, got {:?}", frame_register),
        ))?;

        let start = 2 + len_offset;
        let end = {
            match frame_register {
                FrameRegisters::ReadInt8
                | FrameRegisters::ReadInt16
                | FrameRegisters::ReadInt32
                | FrameRegisters::ReadF32 => start,
                _ => (len as usize * index_step) + 2 + len_offset,
            }
        };
        let data = {
            let mut data = Vec::new();
            for (reg_index, i) in (start..end).step_by(index_step).enumerate() {
                let reg_addr = (initial_reg + reg_index as u8) as u16; //TODO: no bad u8 must be u16
                let res = frame_register.resolution().ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Expected a Reply FrameRegister, got {:?}", frame_register),
                ))?;

                let reg = RegisterDataStruct::from_bytes(reg_addr, &buf[i..i + index_step], res)
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Unable to parse register from {}: {:?}", reg_addr, e),
                        )
                    })?;
                data.push(reg);
                //
                // let value = match frame_register {
                //     FrameRegisters::ReplyInt8 => Data::Int8(i8::from_le_bytes(
                //         buf[i..i + index_step].try_into().unwrap(),
                //     )),
                //     FrameRegisters::ReplyInt16 => Data::Int16(i16::from_le_bytes(
                //         buf[i..i + index_step].try_into().unwrap(),
                //     )),
                //     FrameRegisters::ReplyInt32 => Data::Int32(i32::from_le_bytes(
                //         buf[i..i + index_step].try_into().unwrap(),
                //     )),
                //     FrameRegisters::ReplyF32 => Data::F32(f32::from_le_bytes(
                //         buf[i..i + index_step].try_into().unwrap(),
                //     )),
                //     _ => {
                //         return Err(std::io::Error::new(
                //             std::io::ErrorKind::InvalidData,
                //             format!("Expected a Reply FrameRegister, got {:?}", frame_register),
                //         ));
                //     }
                // };
                // data.push((
                //     // RegisterAddr::from_u8(initial_reg + reg_index as u8).ok_or(
                //     //     std::io::Error::new(
                //     //         std::io::ErrorKind::InvalidData,
                //     //         format!(
                //     //             "Unable to parse register from {}",
                //     //             initial_reg + reg_index as u8
                //     //         ),
                //     //     ),
                //     // )?,
                //     value,
                // ));
            }
            data
        };
        Ok((
            Some(Self {
                register: frame_register,
                len,
                data,
            }),
            end,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct ResponseFrame(Vec<RegisterDataStruct>);

impl ResponseFrame {
    pub(crate) fn from_bytes(buf: &[u8]) -> std::io::Result<ResponseFrame> {
        let mut results = Vec::new();
        let mut buf = buf;
        loop {
            let (subframe, offset) = SubFrame::from_bytes(buf)?;
            if let Some(subframe) = subframe {
                subframe.data.into_iter().for_each(|reg| {
                    results.push(reg);
                });
            }
            buf = &buf[offset..];
            if buf.is_empty() {
                break;
            }
        }
        Ok(ResponseFrame(results))
    }

    pub fn get<R: Register>(&self) -> Option<R> {
        let register = R::address();
        self.0
            .iter()
            .find(|reg| reg.address == register)
            .map(|reg| reg.as_reg::<R>().unwrap())
    }
}

impl TryFrom<CanFdFrame> for ResponseFrame {
    type Error = std::io::Error;

    fn try_from(frame: CanFdFrame) -> Result<Self, Self::Error> {
        let buf = frame.data;
        ResponseFrame::from_bytes(&buf)
    }
}

pub struct Frame {
    subframes: Vec<SubFrame>,
}

impl Frame {
    pub(crate) fn as_bytes(&self) -> Result<Vec<u8>, FrameError> {
        let mut buf = Vec::new();
        for subframe in &self.subframes {
            buf.extend(subframe.as_bytes()?);
        }
        Ok(buf)
    }

    pub fn builder() -> FrameBuilder {
        FrameBuilder {
            registers: HashMap::new(),
        }
    }
}

// impl From<FrameBuilder> for Frame {
//     fn from(builder: FrameBuilder) -> Self {
//         let subframes = builder.subframes;
//         Frame { subframes }
//     }
// }
#[derive(Debug, PartialEq, Clone)]
pub struct FrameBuilder {
    registers: HashMap<FrameRegisters, HashMap<RegisterAddr, RegisterDataStruct>>,
}

impl FrameBuilder {
    pub fn add<R>(self, registers: R) -> Self
    where
        R: IntoIterator<Item = RegisterDataStruct>,
    {
        let new = FrameBuilder::from(registers);
        self.merge(new)
    }

    pub fn add_single(mut self, reg: RegisterDataStruct) -> Self {
        let r = match (reg.resolution, reg.data.is_none()) {
            (Resolution::Int8, true) => FrameRegisters::ReadInt8,
            (Resolution::Int16, true) => FrameRegisters::ReadInt16,
            (Resolution::Int32, true) => FrameRegisters::ReadInt32,
            (Resolution::Float, true) => FrameRegisters::ReadF32,
            (Resolution::Int8, false) => FrameRegisters::WriteInt8,
            (Resolution::Int16, false) => FrameRegisters::WriteInt16,
            (Resolution::Int32, false) => FrameRegisters::WriteInt32,
            (Resolution::Float, false) => FrameRegisters::WriteF32,
        };
        self.registers
            .entry(r)
            .or_default()
            .insert(reg.address, reg);
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        other.registers.into_iter().for_each(|(register, regs)| {
            let Some(existing) = self.registers.get_mut(&register) else {
                self.registers.insert(register, regs);
                return;
            };
            existing.extend(regs);
        });
        self
    }

    pub fn build(self) -> Frame {
        let subframes = self
            .registers
            .into_iter()
            .sorted_by_key(|(k, _)| *k as u8)
            .flat_map(|(frame_register, regs)| {
                let mut subframes = Vec::new();
                let mut regs: Vec<(RegisterAddr, RegisterDataStruct)> = regs.into_iter().collect();
                regs.sort_by_key(|(k, _)| *k as u8);
                let mut regs = regs.into_iter().peekable();
                let mut base_reg = regs.peek().unwrap().0 as u8;
                let mut reg_index = 0;
                let mut subframe = SubFrame::new(frame_register, 0);

                for (reg, value) in regs {
                    if reg as u8 != base_reg + reg_index {
                        subframe.len = reg_index;
                        subframes.push(subframe);
                        reg_index = 0;
                        subframe = SubFrame::new(frame_register, 0);
                        base_reg = reg as u8;
                    }
                    subframe.add(value).unwrap(); //TODO: check unwrap, i dont think this can fail.
                    reg_index += 1;
                }
                subframe.len = reg_index;
                subframes.push(subframe);

                subframes
            })
            .collect();
        Frame { subframes }
    }
}

impl<R> From<R> for FrameBuilder
where
    R: IntoIterator<Item = RegisterDataStruct>,
{
    fn from(registers: R) -> Self {
        let registers: HashMap<FrameRegisters, HashMap<RegisterAddr, RegisterDataStruct>> =
            registers.into_iter().fold(HashMap::new(), |mut acc, reg| {
                let r = match (reg.resolution, reg.data.is_none()) {
                    (Resolution::Int8, true) => FrameRegisters::ReadInt8,
                    (Resolution::Int16, true) => FrameRegisters::ReadInt16,
                    (Resolution::Int32, true) => FrameRegisters::ReadInt32,
                    (Resolution::Float, true) => FrameRegisters::ReadF32,
                    (Resolution::Int8, false) => FrameRegisters::WriteInt8,
                    (Resolution::Int16, false) => FrameRegisters::WriteInt16,
                    (Resolution::Int32, false) => FrameRegisters::WriteInt32,
                    (Resolution::Float, false) => FrameRegisters::WriteF32,
                };
                acc.entry(r).or_default().insert(reg.address, reg);

                acc
            });
        FrameBuilder { registers }
    }
}
//
// impl From<HashMap<FrameRegisters, HashMap<RegisterAddr, Data>>> for FrameBuilder {
//     fn from(registers: HashMap<FrameRegisters, HashMap<RegisterAddr, Data>>) -> Self {
//         let mut builder = FrameBuilder { registers };
//         builder
//     }
// }
//
// impl From<(FrameRegisters, HashSet<RegisterAddr>)> for FrameBuilder {
//     fn from(registers: (FrameRegisters, HashSet<RegisterAddr>)) -> Self {
//         let (frame_reg, registers) = registers;
//         let mut builder = FrameBuilder {
//             registers: HashMap::new(),
//         };
//         builder.registers = registers.into_iter().fold(HashMap::new(), |mut acc, k| {
//             acc.entry(frame_reg)
//                 .or_insert(HashMap::new())
//                 .insert(k, Data::None);
//             acc
//         });
//         builder
//     }
// }
//
// impl From<HashMap<RegisterAddr, Data>> for FrameBuilder {
//     fn from(registers: HashMap<RegisterAddr, Data>) -> Self {
//         let mut builder = FrameBuilder {
//             registers: HashMap::new(),
//         };
//         builder.registers = registers.iter().fold(HashMap::new(), |mut acc, (k, v)| {
//             let r = match v {
//                 Data::Int8(_) => FrameRegisters::WriteInt8,
//                 Data::Int16(_) => FrameRegisters::WriteInt16,
//                 Data::Int32(_) => FrameRegisters::WriteInt32,
//                 Data::F32(_) => FrameRegisters::WriteF32,
//                 Data::None => panic!(), //return Err(FrameError::MixedReadWrites)
//             };
//             acc.entry(r).or_insert(HashMap::new()).insert(*k, *v);
//             acc
//         });
//         builder
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::registers;
    use crate::registers::{Faults, RegisterData};

    #[test]
    fn test_write_u8_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::WriteInt8, 1);
        subframe
            .add(registers::Mode::write(registers::Modes::Stopped).into())
            .unwrap();
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
    fn test_write_u16_subframe() {
        // let mut subframes = Vec::new();
        let mut subframe: SubFrame = SubFrame::new(FrameRegisters::WriteInt16, 3);
        subframe
            .add(registers::CommandPosition::write_with_resolution(2.0, Resolution::Int16).into()) //RegisterAddr::CommandPosition, 0x0060i16)
            .unwrap();
        subframe
            .add(registers::CommandVelocity::write_with_resolution(2.0, Resolution::Int16).into()) //RegisterAddr::CommandPosition, 0x0060i16)
            .unwrap();
        subframe
            .add(
                registers::CommandFeedforwardTorque::write_with_resolution(2.0, Resolution::Int16)
                    .into(),
            ) //RegisterAddr::CommandPosition, 0x0060i16)
            .unwrap();

        let bytes = subframe.as_bytes().unwrap();
        assert_eq!(bytes, vec![0x07, 0x20, 32, 78, 63, 31, 200, 0]);
    }

    #[test]
    fn test_read_u8_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::ReadInt8, 3);
        subframe.add(registers::Voltage::read().into()).unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x13, 0x0d]));
    }

    #[test]
    fn test_read_u16_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::ReadInt16, 4);
        subframe
            .add(registers::Mode::read_with_resolution(Resolution::Int16).into())
            .unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x14, 0x04, 0x00]));
    }

    #[test]
    fn test_read_u32_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::ReadInt32, 5);
        subframe
            .add(registers::MillisecondCounter::read().into())
            .unwrap();
        let bytes = subframe.as_bytes();

        assert_eq!(bytes, Ok(vec![0x18, 0x05, 0x70]));
    }

    #[test]
    fn parse_u8_subframe() {
        // 2404000a005000000170ff230d181400
        let buf = vec![0x23, 0x0d, 0x18, 0x14, 0x00];
        let (subframe, _) = SubFrame::from_bytes(&buf).unwrap();
        let subframe = subframe.unwrap();
        assert_eq!(subframe.register, FrameRegisters::ReplyInt8);
        assert_eq!(subframe.len, 3);
        assert_eq!(
            subframe.data,
            vec![
                registers::Voltage::write_with_resolution(12.0, Resolution::Int8).into(),
                registers::Temperature::write_with_resolution(20.0, Resolution::Int8).into(),
                registers::Fault::write(Faults::Success).into(),
            ]
        );
    }

    #[test]
    fn parse_u16_subframe() {
        // 2404000a005000000170ff230d181400
        let buf = vec![0x24, 0x04, 0x00, 0x0a, 0x00, 100, 0x00, 144, 0x01, 192, 199]; // , 0x23, 0x0d, 0x18, 0x14, 0x00];
        let (subframe, _) = SubFrame::from_bytes(&buf).unwrap();
        let subframe = subframe.unwrap();
        assert_eq!(subframe.register, FrameRegisters::ReplyInt16);
        assert_eq!(subframe.len, 4);
        assert_eq!(
            subframe.data,
            vec![
                // (RegisterAddr::Mode, Data::Int16(10)),
                // (RegisterAddr::Position, Data::Int16(80)),
                // (RegisterAddr::Velocity, Data::Int16(256)),
                // (RegisterAddr::Torque, Data::Int16(-144)),
                registers::Mode::write_with_resolution(
                    registers::Modes::Position,
                    Resolution::Int16
                )
                .into(),
                registers::Position::write_with_resolution(0.01, Resolution::Int16).into(),
                registers::Velocity::write_with_resolution(0.1, Resolution::Int16).into(),
                registers::Torque::write_with_resolution(-144.0, Resolution::Int16).into(),
            ]
        );
    }

    #[test]
    fn parse_multiple_subframes() {
        let frame = SubFrame::from_bytes(&[0x01, 0x00, 0x0a]).unwrap();
        dbg!(&frame);
        let frame =
            SubFrame::from_bytes(&[0x0d, 0x20, 0x00, 0x00, 0xc0, 0x7f, 0x50, 0x50, 0x50]).unwrap();
        dbg!(&frame);
        // assert_eq!(
        //     frame.0,
        //     [
        //         (RegisterAddr::Mode, registers::Mode::read()),
        //         (RegisterAddr::Position, Data::Int16(80)),
        //         (RegisterAddr::Velocity, Data::Int16(256)),
        //         (RegisterAddr::Torque, Data::Int16(-144)),
        //         (RegisterAddr::Voltage, Data::Int8(24)),
        //         (RegisterAddr::Temperature, Data::Int8(20)),
        //         (RegisterAddr::Fault, Data::Int8(0)),
        //     ]
        //         .into()
        // );
    }

    #[test]
    fn multi_subframes_into_bytes() {
        let frame: Frame = Frame::builder()
            .add([
                // (RegisterAddr::CommandPosition, Data::Int16(0x0060)),
                // (RegisterAddr::CommandVelocity, Data::Int16(0x0120)),
                // (RegisterAddr::CommandFeedforwardTorque, Data::Int16(-144)),
                registers::CommandPosition::write_with_resolution(1.0, Resolution::Int16).into(),
                registers::CommandVelocity::write_with_resolution(0.0, Resolution::Int16).into(),
                registers::CommandFeedforwardTorque::write_with_resolution(-2.0, Resolution::Int16)
                    .into(),
                registers::Voltage::read_with_resolution(Resolution::Int8).into(),
                registers::Temperature::read_with_resolution(Resolution::Int8).into(),
                registers::Fault::read_with_resolution(Resolution::Int8).into(),
            ])
            .build();
        let bytes = frame.as_bytes();
        assert_eq!(
            bytes.unwrap(),
            vec![0x07, 0x20, 16, 39, 0, 0, 56, 255, 19, 13]
        );
    }

    // #[test]
    // fn check_packet_diff() {
    //     let a = "01000A0E200000003F000000000E28000040400000004111001F01130D505050";
    //     let b = "01000A0D20000000BF1100130D1F011C0638505050505050";
    //     let a = FdCanUSBFrame::from(format!("rcv 8001 {}\n", a).as_str());
    //     let b = FdCanUSBFrame::from(format!("rcv 8001 {}\n", b).as_str());
    //     let a: CanFdFrame = a.try_into().unwrap();
    //     let b: CanFdFrame = b.try_into().unwrap();
    //     let a: ResponseFrame = a.try_into().unwrap();
    //     let b: ResponseFrame = b.try_into().unwrap();
    //     assert_eq!(a, b);
    // }
    // #[test]
    // fn multi_subframes_into_bytes2() {
    //     let mut builder = FrameBuilder {
    //         registers: HashMap::new()
    //     };
    //     builder = builder.add(Resolution::Int16, vec![
    //         (Register::CommandPosition, Data::Int16(0x0060)),
    //         (Register::CommandVelocity, Data::Int16(0x0120)),
    //         (Register::CommandFeedforwardTorque, Data::Int16(-144)),
    //     ]).unwrap().add(Resolution::Int8, vec![
    //         (Register::Voltage, Data::Int8(24)),
    //         (Register::Temperature, Data::Int8(20)),
    //         (Register::Fault, Data::Int8(0)),
    //     ]).unwrap();
    //     let bytes = builder.as_bytes().unwrap();
    //     assert_eq!(bytes, vec![0x07, 0x20, 0x60, 0x00, 0x20, 0x01, 0x70, 0xff, 0x13, 0x0d]);
    // }
}
