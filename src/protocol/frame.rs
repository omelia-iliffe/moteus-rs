use std::collections::HashMap;

use crate::error::FrameError;
use crate::protocol::registers::{FrameRegisters, RegisterData};
use crate::registers::{Register, RegisterAddr, Res};
use crate::{FrameParseError, Resolution};
use fdcanusb::CanFdFrame;
use itertools::Itertools;
use num_traits::FromPrimitive;

#[derive(Debug, PartialEq)]
pub struct SubFrame {
    register: FrameRegisters,
    len: u8,
    data: Vec<RegisterData>,
}

impl SubFrame {
    pub fn new(register: FrameRegisters, len: u8) -> Self {
        SubFrame {
            register,
            len,
            data: Vec::new(),
        }
    }

    fn add(&mut self, register: RegisterData) -> Result<(), FrameError> {
        if let Some(prev_reg) = self.data.last() {
            if (prev_reg.address as u16) + 1 != register.address as u16 {
                return Err(FrameError::NonSequentialRegisters);
            }
            // TODO: this code does nothing? Im not sure what happened here. Impl a check to see if the register is read or write.
            if register.data.is_none() != register.data.is_none() {
                return Err(FrameError::MixedReadWrites);
            }
        }
        self.data.push(register);
        Ok(())
    }

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
    pub(crate) fn from_bytes(buf: &[u8]) -> Result<(Option<Self>, usize), FrameParseError> {
        if buf.is_empty() {
            return Ok((None, 0));
        }
        let frame_register = buf[0] & (0xFF - 0x03);
        let frame_register = FrameRegisters::from_u8(frame_register)
            .ok_or(FrameParseError::InvalidFrameRegister(frame_register))?;
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
        // todo! added support for read/write error frame registers
        let resolution = frame_register
            .resolution()
            .ok_or(FrameParseError::UnsupportedSubframeRegister(frame_register))?;
        let index_step = resolution.size();
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
                let reg_addr = initial_reg as u16 + reg_index as u16;

                let reg =
                    RegisterData::from_bytes(reg_addr, &buf[i..i + index_step], resolution)?;
                data.push(reg);
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

/// A response frame is a collection of registers returned from the Moteus Controller.
/// The registers can be accessed by their type using the `get` method.
/// Many registers can be accessed at once using the `get_many` method.
#[derive(Debug, PartialEq)]
pub struct ResponseFrame(Vec<RegisterData>);

impl ResponseFrame {
    pub(crate) fn from_bytes(buf: &[u8]) -> Result<ResponseFrame, FrameParseError> {
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

    /// Get a register from the response frame
    /// If the register `R` is not found in the response frame [`None`] is returned.
    pub fn get<R: Register>(&self) -> Option<Res<R>> {
        let register = R::address();
        self.0
            .iter()
            .find(|reg| reg.address == register)
            .and_then(|reg| reg.as_res::<R>().ok())
    }

    /// Get many registers from the response frame
    /// If any of the registers are not found in the response frame [`None`] is returned.
    pub fn get_many<F: FnOnce(&ResponseFrame) -> Option<R>, R>(&self, f: F) -> Option<R> {
        f(self)
    }
}

impl TryFrom<CanFdFrame> for ResponseFrame {
    type Error = FrameParseError;

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

    /// Quickly create and alter a [`FrameBuilder`] with a closure
    /// ### Example
    ///
    ///```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use moteus::frame::QueryType;
    /// # use moteus::registers::Readable;
    /// let query = QueryType::DefaultAnd(moteus::Frame::with_builder(|b| {
    ///     b.add(moteus::registers::Fault::read())
    ///         .add(moteus::registers::HomeState::read());
    /// }));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_builder(f: impl FnOnce(&mut FrameBuilder) -> ()) -> FrameBuilder {
        let mut builder = Frame::builder();
        f(&mut builder);
        builder
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
    registers: HashMap<FrameRegisters, HashMap<RegisterAddr, RegisterData>>,
}

impl FrameBuilder {
    fn frame_register(resolution: Resolution, read: bool) -> FrameRegisters {
        match (resolution, read) {
            (Resolution::Int8, true) => FrameRegisters::ReadInt8,
            (Resolution::Int16, true) => FrameRegisters::ReadInt16,
            (Resolution::Int32, true) => FrameRegisters::ReadInt32,
            (Resolution::Float, true) => FrameRegisters::ReadF32,
            (Resolution::Int8, false) => FrameRegisters::WriteInt8,
            (Resolution::Int16, false) => FrameRegisters::WriteInt16,
            (Resolution::Int32, false) => FrameRegisters::WriteInt32,
            (Resolution::Float, false) => FrameRegisters::WriteF32,
        }
    }

    /// Use a closure that returns [`crate::Error`] to add multiple registers to the frame. Used when constructing write registers.
    ///
    /// ### Example
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use moteus::*;
    /// # use moteus::registers::Writeable;
    /// let mut builder = Frame::builder();
    /// builder.try_add_many(|b| {
    ///     b.add(registers::Mode::write(registers::Modes::Position)?)
    ///      .add(registers::CommandPosition::write(0.0)?);
    ///     Ok(())
    /// })?;
    /// builder.build();
    /// # Ok(())
    /// # }
    pub fn try_add_many(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<(), crate::Error>,
    ) -> Result<&mut Self, crate::Error> {
        f(self)?;
        Ok(self)
    }

    /// Add a single register to the frame
    pub fn add(&mut self, reg: impl Into<RegisterData>) -> &mut Self {
        let reg = reg.into();
        let r = FrameBuilder::frame_register(reg.resolution, reg.data.is_none());
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
                let mut regs: Vec<(RegisterAddr, RegisterData)> = regs.into_iter().collect();
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
    R: IntoIterator<Item = RegisterData>,
{
    fn from(registers: R) -> Self {
        let registers: HashMap<FrameRegisters, HashMap<RegisterAddr, RegisterData>> =
            registers.into_iter().fold(HashMap::new(), |mut acc, reg| {
                let r = FrameBuilder::frame_register(reg.resolution, reg.data.is_none());
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
    use crate::registers::{Faults, Readable, Writeable};

    #[test]
    fn test_write_u8_subframe() {
        let mut subframe = SubFrame::new(FrameRegisters::WriteInt8, 1);
        subframe
            .add(
                registers::Mode::write(registers::Modes::Stopped)
                    .unwrap()
                    .into(),
            )
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
            .add(
                registers::CommandPosition::write_with_resolution(2.0, Resolution::Int16)
                    .unwrap()
                    .into(),
            )
            .expect("Failed to add register");
        subframe
            .add(
                registers::CommandVelocity::write_with_resolution(2.0, Resolution::Int16)
                    .unwrap()
                    .into(),
            )
            .expect("Failed to add register");
        subframe
            .add(
                registers::CommandFeedforwardTorque::write_with_resolution(2.0, Resolution::Int16)
                    .unwrap()
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
                registers::Voltage::write_with_resolution(12.0, Resolution::Int8)
                    .unwrap()
                    .into(),
                registers::Temperature::write_with_resolution(20.0, Resolution::Int8)
                    .unwrap()
                    .into(),
                registers::Fault::write(Faults::Success).unwrap().into(),
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
                .unwrap()
                .into(),
                registers::Position::write_with_resolution(0.01, Resolution::Int16)
                    .unwrap()
                    .into(),
                registers::Velocity::write_with_resolution(0.1, Resolution::Int16)
                    .unwrap()
                    .into(),
                registers::Torque::write_with_resolution(-144.0, Resolution::Int16)
                    .unwrap()
                    .into(),
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
            frame.get::<registers::Mode>().unwrap(),
            registers::Mode::write(registers::Modes::Position).unwrap()
        ); // type returned from frame.get() is inferred.
        assert_eq!(
            frame.get::<registers::CommandPosition>().map(|r| r.value()),
            Some(1.0)
        ); //use the turbofish syntax when the type cannot be inferred.
    }

    #[test]
    fn multi_subframes_into_bytes() {
        let mut builder = Frame::builder();

        builder
            .try_add_many(|f| {
                f.add(registers::CommandPosition::write_with_resolution(
                    1.0,
                    Resolution::Int16,
                )?)
                .add(registers::CommandVelocity::write_with_resolution(
                    0.0,
                    Resolution::Int16,
                )?)
                .add(registers::CommandFeedforwardTorque::write_with_resolution(
                    -2.0,
                    Resolution::Int16,
                )?)
                .add(registers::Voltage::read_with_resolution(Resolution::Int8))
                .add(registers::Temperature::read_with_resolution(
                    Resolution::Int8,
                ))
                .add(registers::Fault::read_with_resolution(Resolution::Int8));
                Ok(())
            })
            .unwrap();
        let frame = builder.build();
        let bytes = frame.as_bytes().expect("Unable to convert frame to bytes");
        assert_eq!(bytes, vec![0x07, 0x20, 16, 39, 0, 0, 56, 255, 19, 13]);
    }
}
