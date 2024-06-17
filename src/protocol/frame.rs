use std::collections::HashMap;

use fdcanusb::CanFdFrame;
use itertools::Itertools;
use num_traits::FromPrimitive;

use crate::protocol::registers::{FrameRegisters, RegisterDataStruct, RegisterError};
use crate::registers::{Register, RegisterAddr};
use crate::Resolution;

#[derive(Debug)]
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
                buf.extend_from_slice(reg.data.as_ref().expect("Reg has no data").as_slice());
            });
        }

        Ok(buf)
    }

    /// Return the parsed subframe and the number of bytes consumed
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
            .and_then(|reg| reg.as_reg::<R>().ok())
    }
}

impl TryFrom<CanFdFrame> for ResponseFrame {
    type Error = std::io::Error;

    fn try_from(frame: CanFdFrame) -> Result<Self, Self::Error> {
        let buf = frame.data;
        ResponseFrame::from_bytes(&buf)
    }
}

/// A frame is a collection of subframes
/// These can be converted into bytes and sent to the Moteus Controller.
#[derive(Debug, PartialEq)]
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

    /// As building frames with multiple resolutions and read/write operations is complex,
    /// a [`FrameBuilder`] is provided to simplify the process.
    pub fn builder() -> FrameBuilder {
        FrameBuilder {
            registers: HashMap::new(),
        }
    }
}

/// A builder for creating a [`Frame`].
/// This is the recommended way to create a frame.
///
/// Registers can be added in any order, and the builder will sort them into subframes.
/// Multiple [`FrameBuilder`]s can be merged together.
/// Duplicate registers are overwritten without warning.
#[derive(Debug, PartialEq, Clone)]
pub struct FrameBuilder {
    registers: HashMap<FrameRegisters, HashMap<RegisterAddr, RegisterDataStruct>>,
}

impl FrameBuilder {
    /// Add multiple register to the frame
    ///
    /// ### Example
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use moteus::*;
    /// use registers::RegisterData;
    /// let frame = Frame::builder().add([registers::Mode::write(registers::Modes::Position).into(), registers::CommandPosition::write(0.0).into()]).build();
    /// # Ok(())
    /// # }
    pub fn add<R>(self, registers: R) -> Self
        where
            R: IntoIterator<Item=RegisterDataStruct>,
    {
        let new = FrameBuilder::from(registers);
        self.merge(new)
    }

    /// Add a single register to the frame
    pub fn add_single(mut self, reg: impl Into<RegisterDataStruct>) -> Self {
        let reg = reg.into();
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
        let _ = self
            .registers
            .entry(r)
            .or_default()
            .insert(reg.address, reg);
        self
    }

    /// Merge two [`FrameBuilder`]s together
    pub fn merge(mut self, other: Self) -> Self {
        other.registers.into_iter().for_each(|(register, regs)| {
            let Some(existing) = self.registers.get_mut(&register) else {
                let _ = self.registers.insert(register, regs);
                return;
            };
            existing.extend(regs);
        });
        self
    }

    #[allow(clippy::unwrap_used)]
    /// Build the frame
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
                let mut base_reg = regs.peek().unwrap().0 as u8; // This `unwrap()` cannot fail when using pub API
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
                    subframe.add(value).unwrap(); // This `unwrap()` cannot fail when using pub API
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
        R: IntoIterator<Item=RegisterDataStruct>,
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
                let _ = acc.entry(r).or_default().insert(reg.address, reg);

                acc
            });
        FrameBuilder { registers }
    }
}

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
            .expect("Failed to add register");
        let bytes = subframe
            .as_bytes()
            .expect("Unable to convert frame to bytes");

        assert_eq!(bytes, vec![0x01, 0x00, 0x00]);
    }

    #[test]
    fn test_empty_subframe() {
        let subframe = SubFrame::new(FrameRegisters::WriteInt8, 1);
        assert!(subframe.as_bytes().is_err());
    }

    #[test]
    fn test_write_u16_subframe() {
        let mut subframe: SubFrame = SubFrame::new(FrameRegisters::WriteInt16, 3);
        subframe
            .add(registers::CommandPosition::write_with_resolution(2.0, Resolution::Int16).into())
            .expect("Failed to add register");
        subframe
            .add(registers::CommandVelocity::write_with_resolution(2.0, Resolution::Int16).into())
            .expect("Failed to add register");
        subframe
            .add(
                registers::CommandFeedforwardTorque::write_with_resolution(2.0, Resolution::Int16)
                    .into(),
            )
            .expect("Failed to add register");

        let bytes = subframe
            .as_bytes()
            .expect("Failed to convert subframe to bytes");
        assert_eq!(bytes, vec![0x07, 0x20, 32, 78, 63, 31, 200, 0]);
    }

    #[test]
    fn test_read_u8_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::ReadInt8, 3);
        subframe
            .add(registers::Voltage::read().into())
            .expect("Failed to add register");
        let bytes = subframe
            .as_bytes()
            .expect("Unable to convert frame to bytes");

        assert_eq!(bytes, vec![0x13, 0x0d]);
    }

    #[test]
    fn test_read_u16_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::ReadInt16, 4);
        subframe
            .add(registers::Mode::read_with_resolution(Resolution::Int16).into())
            .expect("Failed to add register");
        let bytes = subframe
            .as_bytes()
            .expect("Unable to convert frame to bytes");

        assert_eq!(bytes, vec![0x14, 0x04, 0x00]);
    }

    #[test]
    fn test_read_u32_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::ReadInt32, 5);
        subframe
            .add(registers::MillisecondCounter::read().into())
            .expect("Failed to add register");
        let bytes = subframe
            .as_bytes()
            .expect("Unable to convert frame to bytes");

        assert_eq!(bytes, vec![0x18, 0x05, 0x70]);
    }

    #[test]
    fn parse_u8_subframe() {
        // 2404000a005000000170ff230d181400
        let buf = vec![0x23, 0x0d, 0x18, 0x14, 0x00];
        let (subframe, _) = SubFrame::from_bytes(&buf).expect("Failed to parse subframe");
        let subframe = subframe.expect("Failed to parse subframe");
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
        let (subframe, _) = SubFrame::from_bytes(&buf).expect("Failed to parse subframe");
        let subframe = subframe.expect("Failed to parse subframe");
        assert_eq!(subframe.register, FrameRegisters::ReplyInt16);
        assert_eq!(subframe.len, 4);
        assert_eq!(
            subframe.data,
            vec![
                registers::Mode::write_with_resolution(
                    registers::Modes::Position,
                    Resolution::Int16,
                )
                    .into(),
                registers::Position::write_with_resolution(0.01, Resolution::Int16).into(),
                registers::Velocity::write_with_resolution(0.1, Resolution::Int16).into(),
                registers::Torque::write_with_resolution(-144.0, Resolution::Int16).into(),
            ]
        );
    }

    #[test]
    fn parse_response_frame() {
        let buf = vec![
            0x01, 0x00, 0x0a, 0x0d, 0x20, 0x00, 0x00, 0x80, 0x3f, 0x50, 0x50, 0x50,
        ];
        let frame = ResponseFrame::from_bytes(&buf).expect("Failed to parse response frame");
        assert_eq!(
            frame.get(),
            Some(registers::Mode::write(registers::Modes::Position))
        ); // type returned from frame.get() is inferred.
        assert_eq!(
            frame
                .get::<registers::CommandPosition>()
                .and_then(|r| r.value()),
            Some(1.0)
        ); //use the turbofish syntax when the type cannot be inferred.
    }

    #[test]
    fn multi_subframes_into_bytes() {
        let frame: Frame = Frame::builder()
            .add([
                registers::CommandPosition::write_with_resolution(1.0, Resolution::Int16).into(),
                registers::CommandVelocity::write_with_resolution(0.0, Resolution::Int16).into(),
                registers::CommandFeedforwardTorque::write_with_resolution(-2.0, Resolution::Int16)
                    .into(),
                registers::Voltage::read_with_resolution(Resolution::Int8).into(),
                registers::Temperature::read_with_resolution(Resolution::Int8).into(),
                registers::Fault::read_with_resolution(Resolution::Int8).into(),
            ])
            .build();
        let bytes = frame.as_bytes().expect("Unable to convert frame to bytes");
        assert_eq!(bytes, vec![0x07, 0x20, 16, 39, 0, 0, 56, 255, 19, 13]);
    }
}
