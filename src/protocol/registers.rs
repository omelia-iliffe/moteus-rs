//! Registers for the Moteus controllers
//!
//! The Moteus controllers have a number of registers that can be read from and written to. This module provides a number of traits and structs to help with reading and writing to these registers.
//! A list of registers can be found in the [Moteus Reference](https://github.com/mjbots/moteus/blob/main/docs/reference.md#a2b-registers).
//!
//! This module contains the register structs as well as trait interfaces and register types (such as [`Modes`] and [`HomeStates`]).

use crate::{RegisterError, Resolution};
use byteorder::{ReadBytesExt, LE};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use zerocopy::AsBytes;

/// Used to define a register with Integers as the representation
macro_rules! int_rw_register {
    (@IMPL_REG, $reg:ident : $addr:expr, $type:ty, $res:expr) => {
        impl $reg {
            /// If the instance has a value, return it. Otherwise, return None
            pub fn value(&self) -> Option<$type> {
                self.value
            }
            /// Return the resolution
            /// This either is the resolution to be read from the register or the resolution of the value field
            pub fn resolution(&self) -> Resolution {
                self.resolution
            }

            /// Each struct has a default [`Resolution`] that is used when writing to the register.
            const DEFAULT_RESOLUTION: Resolution = $res;

            /// Creates a new instance of the struct with the data to be written using the default resolution.
            pub fn write(data: $type) -> Self {
                $reg {
                    value: Some(data),
                    resolution: Self::DEFAULT_RESOLUTION,
                }
            }
            /// Creates a new instance of the struct with the data to be written using the specified resolution.
            pub fn write_with_resolution(data: $type, r: Resolution) -> Self {
                $reg {
                    value: Some(data),
                    resolution: r,
                }
            }
            /// Creates a new instance of the struct for reading using the default resolution.
            pub fn read() -> Self {
                $reg {
                    value: None,
                    resolution: Self::DEFAULT_RESOLUTION,
                }
            }
            /// Creates a new instance of the struct for reading using the specified resolution.
            pub fn read_with_resolution(r: Resolution) -> Self {
                $reg {
                    value: None,
                    resolution: r,
                }
            }
        }
    };
    (@IMPL_REG_AS_BYTES, $reg:ident : $mapping:expr) => {
        impl $reg {
            fn as_bytes(&self) -> Result<Vec<u8>, RegisterError> {
                let Some(value) = self.value else {
                    return Err(RegisterError::NoData);
                };
                match self.resolution {
                    Resolution::Int8 => value.try_into_1_byte($mapping.0).map(|x| vec![x]),
                    Resolution::Int16 => value.try_into_2_bytes($mapping.1).map(|x| x.to_vec()),
                    Resolution::Int32 => value.try_into_4_bytes($mapping.2).map(|x| x.to_vec()),
                    Resolution::Float => {
                        value.try_into_f32_bytes().map(|x| x.to_vec())
                    }
                }
            }
        }
    };
    (@FROM_DATA_STRUCT, $reg:ident : $type:ty) => {
        impl From<$reg> for RegisterDataStruct {
            fn from(reg: $reg) -> RegisterDataStruct {
                if let Ok(data) = reg.as_bytes() {
                    return RegisterDataStruct {
                        address: $reg::address(),
                        resolution: reg.resolution,
                        data: Some(data),
                    };
                } else {
                    return RegisterDataStruct {
                        address: $reg::address(),
                        resolution: reg.resolution,
                        data: None,
                    };
                }
            }
        }
    };
    (@IMPL_REGISTER, $reg:ident : $addr:expr, $type:ty, $res:expr, $mapping:expr) => {
        impl Register for $reg {
            type INNER = $type;
            const DEFAULT_RESOLUTION: Resolution = $res;
            const MAPPING: Map = $mapping;

            fn address() -> RegisterAddr {
                $addr
            }

            fn from_bytes(bytes: &[u8], resolution: Resolution) -> Result<Self, RegisterError>
            where
                Self: Sized,
            {
                Ok(match resolution {
                    Resolution::Int8 => Self {
                        value: Some(<$type>::try_from_1_byte(bytes[0], $mapping.0)?),
                        resolution,
                    },
                    Resolution::Int16 => Self {
                        value: Some(<$type>::try_from_2_bytes(&bytes[..2], $mapping.1)?),
                        resolution,
                    },
                    Resolution::Int32 => Self {
                        value: Some(<$type>::try_from_4_bytes(&bytes[..4], $mapping.2)?),
                        resolution,
                    },
                    Resolution::Float => Self {
                        value: Some(<$type>::try_from_f32_bytes(&bytes[..4])?),
                        resolution,
                    },
                })
            }
        }
    };
    (@INTERNAL, $reg:ident : $addr:expr, $type:ty, $res:expr, $mapping:expr) => {
        int_rw_register!(@IMPL_REG, $reg : $addr, $type, $res);
        int_rw_register!(@IMPL_REG_AS_BYTES, $reg : $mapping);

        int_rw_register!(@FROM_DATA_STRUCT, $reg : $type);

        int_rw_register!(@IMPL_REGISTER, $reg : $addr, $type, $res, $mapping);
    };
    ($reg:ident : $addr:expr, $type:ty, $res:expr) => {
        #[doc = concat!("Struct representing the ",stringify!($reg)," register at ",stringify!($addr)," .")]
        #[doc = concat!(stringify!($reg)," can be represented as larger ints but not floats or smaller ints")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $reg {
            /// The value of the register
            pub value: Option<$type>,
            /// The resolution of the value
            pub resolution: Resolution,
        }

        int_rw_register!(@INTERNAL, $reg : $addr, $type, $res, NO_MAP);

    };

}

/// Used to define a register with f32 as the representation.
/// These registers using a `Map` to convert to different resolutions
macro_rules! map_rw_register {
    ($reg:ident : $addr:expr, $type:ty, $res:expr, $mapping:expr) => {
        #[derive(Clone, Debug, PartialEq)]
        #[doc = concat!("Struct representing the ",stringify!($reg)," register at ",stringify!($addr)," .")]
        #[doc = concat!(stringify!($reg)," uses `", stringify!($mapping), "` to map between different resolutions")]
        pub struct $reg {
            value: Option<$type>,
            resolution: Resolution,
        }

        int_rw_register!(@INTERNAL, $reg : $addr, $type, $res, $mapping);

    };
    ($reg:ident : $addr:expr, $mapping:expr) => {
       map_rw_register!($reg : $addr, f32, Resolution::Float, $mapping);
    };
}
/// As the Moteus Registers are each a unique struct, they all implement the [`Register`] trait.
pub trait Register {
    /// The inner type of the register
    type INNER;
    /// Each struct has a default [`Resolution`] that is used when writing to the register.
    const DEFAULT_RESOLUTION: Resolution;
    /// The mapping used to
    const MAPPING: Map;
    /// Returns the address of the register as a [`RegisterAddr`].
    fn address() -> RegisterAddr;
    /// Creates the register from a slice of bytes.
    fn from_bytes(bytes: &[u8], resolution: Resolution) -> Result<Self, RegisterError>
    where
        Self: Sized;
}

/// A struct that represents the raw data (as `Vec<u8>`) that has been read from, or will be written to, a register
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RegisterDataStruct {
    /// The [`RegisterAddr`] of the register
    pub address: RegisterAddr,
    /// The [`Resolution`] of the data
    pub resolution: Resolution,
    /// The data to be written to the register, or None if it will be read from the register
    pub data: Option<Vec<u8>>,
}

impl RegisterDataStruct {
    pub(crate) fn as_reg<R: Register>(&self) -> Result<R, RegisterError> {
        let bytes = self.data.as_ref().ok_or(RegisterError::NoData)?;
        R::from_bytes(bytes, self.resolution)
    }

    pub(crate) fn from_bytes(
        addr: u16,
        bytes: &[u8],
        resolution: Resolution,
    ) -> Result<RegisterDataStruct, RegisterError> {
        Ok(RegisterDataStruct {
            address: RegisterAddr::from_u16(addr).ok_or(RegisterError::InvalidAddress)?,
            resolution,
            data: Some(bytes.into()),
        })
    }
}

impl std::fmt::Debug for RegisterDataStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(data) = &self.data {
            write!(f, "{:?}{:?}", &self.address, &data)
        } else {
            write!(f, "{:?}", &self.address)
        }
    }
}

/// A sequence of one or more uint8 values, in least significant byte first order.
/// For each value, the 7 LSBs contain data and if the MSB is set, it means there are more bytes remaining.
/// At most, it may represent a single uint32 and thus 5 bytes is the maximum valid length.
pub type Varuint = Vec<u8>;

impl RegisterAddr {
    /// Converts the address to a [`Varuint`]
    pub fn address_as_bytes(&self) -> Varuint {
        let mut buf = Vec::new();
        let mut val = *self as u16;
        loop {
            let mut this_byte: u8 = (val & 0x7F) as u8;
            val >>= 7;
            this_byte |= if val != 0 { 0x80 } else { 0x00 };
            buf.push(this_byte);

            if val == 0 {
                break;
            }
        }
        buf
    }
}

trait TryIntoBytes {
    fn try_into_1_byte(self, scale: f32) -> Result<u8, RegisterError>;
    fn try_into_2_bytes(self, scale: f32) -> Result<[u8; 2], RegisterError>;
    fn try_into_4_bytes(self, scale: f32) -> Result<[u8; 4], RegisterError>;
    fn try_into_f32_bytes(self) -> Result<[u8; 4], RegisterError>;
}

trait TryFromBytes {
    fn try_from_1_byte(byte: u8, scale: f32) -> Result<Self, RegisterError>
    where
        Self: Sized;
    fn try_from_2_bytes(bytes: &[u8], scale: f32) -> Result<Self, RegisterError>
    where
        Self: Sized;
    fn try_from_4_bytes(bytes: &[u8], scale: f32) -> Result<Self, RegisterError>
    where
        Self: Sized;
    fn try_from_f32_bytes(bytes: &[u8]) -> Result<Self, RegisterError>
    where
        Self: Sized;
}

pub(crate) type Map = (f32, f32, f32);

pub(crate) const NO_MAP: Map = (1.0, 1.0, 1.0);
pub(crate) const POSITION_MAP: Map = (0.01, 0.0001, 0.00001);
pub(crate) const VELOCITY_MAP: Map = (0.1, 0.00025, 0.00001);
pub(crate) const ACCEL_MAP: Map = (0.1, 0.00025, 0.00001);
pub(crate) const TORQUE_MAP: Map = (0.5, 0.01, 0.001);
pub(crate) const PWM_MAP: Map = (1.0 / 127.0, 1.0 / 32767.0, 1.0 / 2147483647.0);
pub(crate) const VOLTAGE_MAP: Map = (0.5, 0.1, 0.001);
pub(crate) const TEMPERATURE_MAP: Map = (1.0, 0.1, 0.001);
#[allow(unused)]
pub(crate) const TIME_MAP: Map = (0.01, 0.001, 0.000001);
pub(crate) const CURRENT_MAP: Map = (1.0, 0.1, 0.001);

/// [`FrameRegisters`] are used to specify the type of data that is being written to or read from a register.
/// Some, like [`FrameRegisters::ReplyInt8`] and [`FrameRegisters::WriteError`], are only returned in responses.
/// Others, like [`FrameRegisters::WriteInt16`] and [`FrameRegisters::ReadF32`], are used when sending frame.
///
/// The number of values can be encoded into the 2 Least Significant bits of the [`FrameRegisters`]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq, AsBytes, Hash)]
#[repr(u8)]
pub enum FrameRegisters {
    WriteInt8 = 0x00,
    WriteInt16 = 0x04,
    WriteInt32 = 0x08,
    WriteF32 = 0x0c,
    // READ_BASE = 0x10,
    ReadInt8 = 0x10,
    ReadInt16 = 0x14,
    ReadInt32 = 0x18,
    ReadF32 = 0x1c,
    // ReplyBase = 0x20,
    ReplyInt8 = 0x20,
    ReplyInt16 = 0x24,
    ReplyInt32 = 0x28,
    ReplyF32 = 0x2c,
    /// returned when writing to a register fails
    WriteError = 0x30,
    /// returned when reading from a register fails
    ReadError = 0x31,
    /// Used to receive ascii data with moteus_tool or tview
    StreamClientData = 0x40,
    /// Used by the moteus_tool or Tview to send ascii data
    StreamServerData = 0x41,
    /// Used by the moteus_tool and Tview to poll for responses
    StreamClientPoll = 0x42,
    /// Used to buffer the can frame
    Nop = 0x50,
}

impl FrameRegisters {
    /// Returns the [`Resolution`] of the register
    pub fn resolution(&self) -> Option<Resolution> {
        let r = match self {
            FrameRegisters::WriteInt8 | FrameRegisters::ReadInt8 | FrameRegisters::ReplyInt8 => {
                Resolution::Int8
            }
            FrameRegisters::WriteInt16 | FrameRegisters::ReadInt16 | FrameRegisters::ReplyInt16 => {
                Resolution::Int16
            }
            FrameRegisters::WriteInt32 | FrameRegisters::ReadInt32 | FrameRegisters::ReplyInt32 => {
                Resolution::Int32
            }
            FrameRegisters::WriteF32 | FrameRegisters::ReadF32 | FrameRegisters::ReplyF32 => {
                Resolution::Float
            }
            _ => return None,
        };
        Some(r)
    }
}

/// Each register of the moteus board has an address which can be encoded as a [`Varuint`]
#[derive(Debug, Clone, Copy, AsBytes, FromPrimitive, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
#[repr(u16)]
pub enum RegisterAddr {
    Mode = 0x000,
    Position = 0x001,
    Velocity = 0x002,
    Torque = 0x003,
    QCurrent = 0x004,
    DCurrent = 0x005,
    AbsPosition = 0x006,

    MotorTemperature = 0x00a,
    TrajectoryComplete = 0x00b,
    HomeState = 0x00c,
    Voltage = 0x00d,
    Temperature = 0x00e,
    Fault = 0x00f,

    PwmPhaseA = 0x010,
    PwmPhaseB = 0x011,
    PwmPhaseC = 0x012,

    VoltagePhaseA = 0x014,
    VoltagePhaseB = 0x015,
    VoltagePhaseC = 0x016,

    VfocTheta = 0x018,
    VfocVoltage = 0x019,
    VoltageDqD = 0x01a,
    VoltageDqQ = 0x01b,

    CommandQCurrent = 0x01c,
    CommandDCurrent = 0x01d,

    CommandPosition = 0x020,
    CommandVelocity = 0x021,
    CommandFeedforwardTorque = 0x022,
    CommandKpScale = 0x023,
    CommandKdScale = 0x024,
    CommandPositionMaxTorque = 0x025,
    CommandStopPosition = 0x026,
    CommandTimeout = 0x027,
    VelocityLimit = 0x028,
    AccelerationLimit = 0x029,
    FixedVoltageOverride = 0x02a,

    PositionKp = 0x030,
    PositionKi = 0x031,
    PositionKd = 0x032,
    PositionFeedforward = 0x033,
    PositionCommand = 0x034,

    ControlPosition = 0x038,
    ControlVelocity = 0x039,
    ControlTorque = 0x03a,
    ControlPositionError = 0x03b,
    ControlVelocityError = 0x03c,
    ControlTorqueError = 0x03d,

    CommandStayWithinLowerBound = 0x040,
    CommandStayWithinUpperBound = 0x041,
    CommandStayWithinFeedforwardTorque = 0x042,
    CommandStayWithinKpScale = 0x043,
    CommandStayWithinKdScale = 0x044,
    CommandStayWithinPositionMaxTorque = 0x045,
    CommandStayWithinTimeout = 0x046,

    Encoder0position = 0x050,
    Encoder0velocity = 0x051,
    Encoder1position = 0x052,
    Encoder1velocity = 0x053,
    Encoder2position = 0x054,
    Encoder2velocity = 0x055,

    EncoderValidity = 0x058,

    Aux1gpioCommand = 0x05c,
    Aux2gpioCommand = 0x05d,
    Aux1gpioStatus = 0x05e,
    Aux2gpioStatus = 0x05f,

    Aux1analogIn1 = 0x060,
    Aux1analogIn2 = 0x061,
    Aux1analogIn3 = 0x062,
    Aux1analogIn4 = 0x063,
    Aux1analogIn5 = 0x064,

    Aux2analogIn1 = 0x068,
    Aux2analogIn2 = 0x069,
    Aux2analogIn3 = 0x06a,
    Aux2analogIn4 = 0x06b,
    Aux2analogIn5 = 0x06c,

    MillisecondCounter = 0x070,
    ClockTrim = 0x071,

    RegisterMapVersion = 0x102,
    SerialNumber = 0x120,
    // SerialNumber1 = 0x120,
    // SerialNumber2 = 0x121,
    // SerialNumber3 = 0x122,
    Rezero = 0x130,
    // SetOutputNearest = 0x130,
    SetOutputExact = 0x131,
    RequireReindex = 0x132,

    DriverFault1 = 0x140,
    DriverFault2 = 0x141,
}

int_rw_register!(Mode: RegisterAddr::Mode, Modes, Resolution::Int8);
map_rw_register!(Position: RegisterAddr::Position, POSITION_MAP);
map_rw_register!(Velocity: RegisterAddr::Velocity, VELOCITY_MAP);
map_rw_register!(Torque: RegisterAddr::Torque, TORQUE_MAP);
map_rw_register!(QCurrent: RegisterAddr::QCurrent, CURRENT_MAP);
map_rw_register!(DCurrent: RegisterAddr::DCurrent, CURRENT_MAP);
map_rw_register!(AbsPosition: RegisterAddr::AbsPosition, POSITION_MAP);

map_rw_register!(MotorTemperature: RegisterAddr::MotorTemperature, TEMPERATURE_MAP);
int_rw_register!(TrajectoryComplete: RegisterAddr::TrajectoryComplete, i8, Resolution::Int8);
int_rw_register!(HomeState: RegisterAddr::HomeState, HomeStates, Resolution::Int8);
map_rw_register!(Voltage: RegisterAddr::Voltage, VOLTAGE_MAP);
map_rw_register!(Temperature: RegisterAddr::Temperature, TEMPERATURE_MAP);
int_rw_register!(Fault: RegisterAddr::Fault, Faults, Resolution::Int8);

map_rw_register!(PwmPhaseA: RegisterAddr::PwmPhaseA, PWM_MAP);
map_rw_register!(PwmPhaseB: RegisterAddr::PwmPhaseB, PWM_MAP);
map_rw_register!(PwmPhaseC: RegisterAddr::PwmPhaseC, PWM_MAP);

map_rw_register!(VoltagePhaseA: RegisterAddr::VoltagePhaseA, VOLTAGE_MAP);
map_rw_register!(VoltagePhaseB: RegisterAddr::VoltagePhaseB, VOLTAGE_MAP);
map_rw_register!(VoltagePhaseC: RegisterAddr::VoltagePhaseC, VOLTAGE_MAP);

map_rw_register!(VfocTheta: RegisterAddr::VfocTheta, NO_MAP);
map_rw_register!(VfocVoltage: RegisterAddr::VfocVoltage, NO_MAP);
map_rw_register!(VoltageDqD: RegisterAddr::VoltageDqD, NO_MAP);
map_rw_register!(VoltageDqQ: RegisterAddr::VoltageDqQ, NO_MAP);

map_rw_register!(CommandQcurrent: RegisterAddr::CommandQCurrent, CURRENT_MAP);
map_rw_register!(CommandDcurrent: RegisterAddr::CommandDCurrent, CURRENT_MAP);

map_rw_register!(CommandPosition: RegisterAddr::CommandPosition, POSITION_MAP);
map_rw_register!(CommandVelocity: RegisterAddr::CommandVelocity, VELOCITY_MAP);
map_rw_register!(CommandFeedforwardTorque: RegisterAddr::CommandFeedforwardTorque, TORQUE_MAP);
map_rw_register!(CommandKpScale: RegisterAddr::CommandKpScale, TORQUE_MAP);
map_rw_register!(CommandKdScale: RegisterAddr::CommandKdScale, TORQUE_MAP);
map_rw_register!(CommandPositionMaxTorque: RegisterAddr::CommandPositionMaxTorque, TORQUE_MAP);
map_rw_register!(CommandStopPosition: RegisterAddr::CommandStopPosition, POSITION_MAP);
map_rw_register!(CommandTimeout: RegisterAddr::CommandTimeout, NO_MAP);
map_rw_register!(VelocityLimit: RegisterAddr::VelocityLimit, VELOCITY_MAP);
map_rw_register!(AccelerationLimit: RegisterAddr::AccelerationLimit, ACCEL_MAP);
map_rw_register!(FixedVoltage: RegisterAddr::FixedVoltageOverride, VOLTAGE_MAP);

map_rw_register!(PositionKp: RegisterAddr::PositionKp, TORQUE_MAP);
map_rw_register!(PositionKi: RegisterAddr::PositionKi, TORQUE_MAP);
map_rw_register!(PositionKd: RegisterAddr::PositionKd, TORQUE_MAP);
map_rw_register!(PositionFeedforward: RegisterAddr::PositionFeedforward, TORQUE_MAP);
map_rw_register!(PositionCommand: RegisterAddr::PositionCommand, TORQUE_MAP);

map_rw_register!(ControlPosition: RegisterAddr::ControlPosition, POSITION_MAP);
map_rw_register!(ControlVelocity: RegisterAddr::ControlVelocity, VELOCITY_MAP);
map_rw_register!(ControlTorque: RegisterAddr::ControlTorque, TORQUE_MAP);
map_rw_register!(ControlPositionError: RegisterAddr::ControlPositionError, POSITION_MAP);
map_rw_register!(ControlVelocityError: RegisterAddr::ControlVelocityError, VELOCITY_MAP);
map_rw_register!(ControlTorqueError: RegisterAddr::ControlTorqueError, TORQUE_MAP);

map_rw_register!(CommandStayWithinLowerBound: RegisterAddr::CommandStayWithinLowerBound, NO_MAP); //TODO: check the mapping
map_rw_register!(CommandStayWithinUpperBound: RegisterAddr::CommandStayWithinUpperBound, NO_MAP);
map_rw_register!(CommandStayWithinFeedforwardTorque: RegisterAddr::CommandStayWithinFeedforwardTorque, NO_MAP);
map_rw_register!(CommandStayWithinKpScale: RegisterAddr::CommandStayWithinKpScale, NO_MAP);
map_rw_register!(CommandStayWithinKdScale: RegisterAddr::CommandStayWithinKdScale, NO_MAP);
map_rw_register!(CommandStayWithinPositionMaxTorque: RegisterAddr::CommandStayWithinPositionMaxTorque, NO_MAP);
map_rw_register!(CommandStayWithinTimeout: RegisterAddr::CommandStayWithinTimeout, NO_MAP);

map_rw_register!(Encoder0position: RegisterAddr::Encoder0position, POSITION_MAP);
map_rw_register!(Encoder0velocity: RegisterAddr::Encoder0velocity, VELOCITY_MAP);
map_rw_register!(Encoder1position: RegisterAddr::Encoder1position, POSITION_MAP);
map_rw_register!(Encoder1velocity: RegisterAddr::Encoder1velocity, VELOCITY_MAP);
map_rw_register!(Encoder2position: RegisterAddr::Encoder2position, POSITION_MAP);
map_rw_register!(Encoder2velocity: RegisterAddr::Encoder2velocity, VELOCITY_MAP);

int_rw_register!(EncoderValidity: RegisterAddr::EncoderValidity, i8, Resolution::Int8);

int_rw_register!(Aux1gpioCommand: RegisterAddr::Aux1gpioCommand, i8, Resolution::Int8);
int_rw_register!(Aux2gpioCommand: RegisterAddr::Aux2gpioCommand, i8, Resolution::Int8);
int_rw_register!(Aux1gpioStatus: RegisterAddr::Aux1gpioStatus, i8, Resolution::Int8);
int_rw_register!(Aux2gpioStatus: RegisterAddr::Aux2gpioStatus, i8, Resolution::Int8);

map_rw_register!(Aux1analogIn1: RegisterAddr::Aux1analogIn1, PWM_MAP);
map_rw_register!(Aux1analogIn2: RegisterAddr::Aux1analogIn2, PWM_MAP);
map_rw_register!(Aux1analogIn3: RegisterAddr::Aux1analogIn3, PWM_MAP);
map_rw_register!(Aux1analogIn4: RegisterAddr::Aux1analogIn4, PWM_MAP);
map_rw_register!(Aux1analogIn5: RegisterAddr::Aux1analogIn5, PWM_MAP);

map_rw_register!(Aux2analogIn1: RegisterAddr::Aux2analogIn1, PWM_MAP);
map_rw_register!(Aux2analogIn2: RegisterAddr::Aux2analogIn2, PWM_MAP);
map_rw_register!(Aux2analogIn3: RegisterAddr::Aux2analogIn3, PWM_MAP);
map_rw_register!(Aux2analogIn4: RegisterAddr::Aux2analogIn4, PWM_MAP);
map_rw_register!(Aux2analogIn5: RegisterAddr::Aux2analogIn5, PWM_MAP);

int_rw_register!(MillisecondCounter: RegisterAddr::MillisecondCounter, i32, Resolution::Int32);
int_rw_register!(ClockTrim: RegisterAddr::ClockTrim, i32, Resolution::Int8);

int_rw_register!(RegisterMapVersion: RegisterAddr::RegisterMapVersion, u32, Resolution::Int32);
int_rw_register!(SerialNumber: RegisterAddr::SerialNumber, u32, Resolution::Int32);
int_rw_register!(Rezero: RegisterAddr::Rezero, i8, Resolution::Int8);
int_rw_register!(SetOutputExact: RegisterAddr::SetOutputExact, i8, Resolution::Int8);
int_rw_register!(RequireReindex: RegisterAddr::RequireReindex, (), Resolution::Int8);

int_rw_register!(DriverFault1: RegisterAddr::DriverFault1, u32, Resolution::Int32);
int_rw_register!(DriverFault2: RegisterAddr::DriverFault2, u32, Resolution::Int32);

impl TryIntoBytes for () {
    fn try_into_1_byte(self, _scale: f32) -> Result<u8, RegisterError> {
        Ok(0)
    }
    fn try_into_2_bytes(self, _scale: f32) -> Result<[u8; 2], RegisterError> {
        Ok([0, 0])
    }
    fn try_into_4_bytes(self, _scale: f32) -> Result<[u8; 4], RegisterError> {
        Ok([0, 0, 0, 0])
    }
    fn try_into_f32_bytes(self) -> Result<[u8; 4], RegisterError> {
        Ok([0, 0, 0, 0])
    }
}

impl TryFromBytes for () {
    fn try_from_1_byte(_: u8, _scale: f32) -> Result<Self, RegisterError> {
        Ok(())
    }
    fn try_from_2_bytes(_: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        Ok(())
    }
    fn try_from_4_bytes(_: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        Ok(())
    }
    fn try_from_f32_bytes(_: &[u8]) -> Result<Self, RegisterError> {
        Ok(())
    }
}

impl TryIntoBytes for i8 {
    fn try_into_1_byte(self, _scale: f32) -> Result<u8, RegisterError> {
        Ok(self as u8)
    }
    fn try_into_2_bytes(self, _scale: f32) -> Result<[u8; 2], RegisterError> {
        Ok((self as i16).to_le_bytes())
    }
    fn try_into_4_bytes(self, _scale: f32) -> Result<[u8; 4], RegisterError> {
        Ok((self as i32).to_le_bytes())
    }
    fn try_into_f32_bytes(self) -> Result<[u8; 4], RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

impl TryFromBytes for i8 {
    fn try_from_1_byte(byte: u8, _scale: f32) -> Result<Self, RegisterError> {
        Ok(byte as i8)
    }
    fn try_from_2_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let value = i16::from_le_bytes([bytes[0], bytes[1]]);
        Ok(value as i8)
    }
    fn try_from_4_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let value = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(value as i8)
    }
    fn try_from_f32_bytes(_: &[u8]) -> Result<Self, RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

impl TryIntoBytes for i32 {
    fn try_into_1_byte(self, _scale: f32) -> Result<u8, RegisterError> {
        let value = self;
        if value > i8::MAX as i32 || value < i8::MIN as i32 {
            return Err(RegisterError::Overflow);
        }
        Ok(value as u8)
    }
    fn try_into_2_bytes(self, _scale: f32) -> Result<[u8; 2], RegisterError> {
        let value = self;
        if value > i16::MAX as i32 || value < i16::MIN as i32 {
            return Err(RegisterError::Overflow);
        }
        Ok((value as i16).to_le_bytes())
    }
    fn try_into_4_bytes(self, _scale: f32) -> Result<[u8; 4], RegisterError> {
        let value = self;
        Ok(value.to_le_bytes())
    }
    fn try_into_f32_bytes(self) -> Result<[u8; 4], RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

impl TryFromBytes for i32 {
    fn try_from_1_byte(byte: u8, _scale: f32) -> Result<Self, RegisterError> {
        let value = byte as i32;
        Ok(value)
    }
    fn try_from_2_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let mut rdr = std::io::Cursor::new(bytes);
        let value = rdr.read_i16::<LE>()?;
        Ok(value as i32)
    }
    fn try_from_4_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let mut rdr = std::io::Cursor::new(bytes);
        let value = rdr.read_i32::<LE>()?;
        Ok(value)
    }
    fn try_from_f32_bytes(bytes: &[u8]) -> Result<Self, RegisterError> {
        let mut rdr = std::io::Cursor::new(bytes);
        let value = rdr.read_f32::<LE>()?;
        Ok(value as i32)
    }
}

impl TryIntoBytes for u32 {
    fn try_into_1_byte(self, _scale: f32) -> Result<u8, RegisterError> {
        let value = self;
        if value > i8::MAX as u32 || value < i8::MIN as u32 {
            return Err(RegisterError::Overflow);
        }
        Ok(value as u8)
    }
    fn try_into_2_bytes(self, _scale: f32) -> Result<[u8; 2], RegisterError> {
        let value = self;
        if value > i16::MAX as u32 || value < i16::MIN as u32 {
            return Err(RegisterError::Overflow);
        }
        Ok((value as u16).to_le_bytes())
    }
    fn try_into_4_bytes(self, _scale: f32) -> Result<[u8; 4], RegisterError> {
        let value = self;
        Ok(value.to_le_bytes())
    }
    fn try_into_f32_bytes(self) -> Result<[u8; 4], RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

impl TryFromBytes for u32 {
    fn try_from_1_byte(byte: u8, _scale: f32) -> Result<Self, RegisterError> {
        let value = byte as u32;
        Ok(value)
    }
    fn try_from_2_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let mut rdr = std::io::Cursor::new(bytes);
        let value = rdr.read_i16::<LE>()?;
        Ok(value as u32)
    }
    fn try_from_4_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let mut rdr = std::io::Cursor::new(bytes);
        let value = rdr.read_i32::<LE>()?;
        Ok(value as u32)
    }
    fn try_from_f32_bytes(_bytes: &[u8]) -> Result<Self, RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

impl TryIntoBytes for f32 {
    fn try_into_1_byte(self, scale: f32) -> Result<u8, RegisterError> {
        if !self.is_finite() {
            return Ok(i8::MIN as u8);
        }
        let value = self / scale;

        if value > i8::MAX as f32 || value < i8::MIN as f32 {
            return Err(RegisterError::Overflow);
        }
        Ok(value as u8)
    }
    fn try_into_2_bytes(self, scale: f32) -> Result<[u8; 2], RegisterError> {
        if !self.is_finite() {
            return Ok(i16::MIN.to_le_bytes());
        }
        let value = self / scale;
        if value > i16::MAX as f32 || value < i16::MIN as f32 {
            return Err(RegisterError::Overflow);
        }
        Ok((value as i16).to_le_bytes())
    }
    fn try_into_4_bytes(self, scale: f32) -> Result<[u8; 4], RegisterError> {
        if !self.is_finite() {
            return Ok(i32::MIN.to_le_bytes());
        }
        let value = self / scale;
        if value > i32::MAX as f32 || value < i32::MIN as f32 {
            return Err(RegisterError::Overflow);
        }
        Ok((value as i32).to_le_bytes())
    }
    fn try_into_f32_bytes(self) -> Result<[u8; 4], RegisterError> {
        let value = self;
        Ok(value.to_le_bytes())
    }
}

impl TryFromBytes for f32 {
    fn try_from_1_byte(byte: u8, scale: f32) -> Result<Self, RegisterError> {
        let value = {
            let int = byte as i8;
            if int == i8::MIN {
                f32::NAN
            } else {
                int as f32
            }
        };

        Ok(value * scale)
    }
    fn try_from_2_bytes(bytes: &[u8], scale: f32) -> Result<Self, RegisterError> {
        let mut rdr = std::io::Cursor::new(bytes);
        let value = rdr.read_i16::<LE>()?;
        let value = {
            if value == i16::MIN {
                f32::NAN
            } else {
                value as f32
            }
        };
        Ok(value * scale)
    }
    fn try_from_4_bytes(bytes: &[u8], scale: f32) -> Result<Self, RegisterError> {
        let mut rdr = std::io::Cursor::new(bytes);
        let value = rdr.read_i32::<LE>()?;
        let value = {
            if value == i32::MIN {
                f32::NAN
            } else {
                value as f32
            }
        };
        Ok(value * scale)
    }
    fn try_from_f32_bytes(bytes: &[u8]) -> Result<Self, RegisterError> {
        let mut rdr = std::io::Cursor::new(bytes);
        let value = rdr.read_f32::<LE>()?;
        Ok(value)
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, AsBytes, FromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum Modes {
    Stopped = 0,
    Fault = 1,
    Enabling = 2,
    Calibrating = 3,
    CalibrationComplete = 4,
    Pwm = 5,
    Voltage = 6,
    VoltageFoc = 7,
    VoltageDq = 8,
    Current = 9,
    Position = 10,
    PositionTimeout = 11,
    ZeroVelocity = 12,
    StayWithin = 13,
    MeasureInd = 14,
    Brake = 15,
}

impl TryIntoBytes for Modes {
    fn try_into_1_byte(self, _scale: f32) -> Result<u8, RegisterError> {
        Ok(self as u8)
    }

    fn try_into_2_bytes(self, _scale: f32) -> Result<[u8; 2], RegisterError> {
        Ok((self as i16).to_le_bytes())
    }

    fn try_into_4_bytes(self, _scale: f32) -> Result<[u8; 4], RegisterError> {
        Ok((self as i32).to_le_bytes())
    }
    fn try_into_f32_bytes(self) -> Result<[u8; 4], RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

impl TryFromBytes for Modes {
    fn try_from_1_byte(byte: u8, _scale: f32) -> Result<Self, RegisterError> {
        Modes::from_u8(byte).ok_or(RegisterError::InvalidData)
    }
    fn try_from_2_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let value = u16::from_le_bytes([bytes[0], bytes[1]]);
        Modes::from_u16(value).ok_or(RegisterError::InvalidData)
    }
    fn try_from_4_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Modes::from_u32(value).ok_or(RegisterError::InvalidData)
    }
    fn try_from_f32_bytes(_bytes: &[u8]) -> Result<Self, RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

#[derive(Debug, Clone, Copy, AsBytes, FromPrimitive, PartialEq, Eq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Faults {
    Success = 0,

    DmaStreamTransferError = 1,
    DmaStreamFifoError = 2,
    UartOverrunError = 3,
    UartFramingError = 4,
    UartNoiseError = 5,
    UartBufferOverrunError = 6,
    UartParityError = 7,

    CalibrationFault = 32,
    MotorDriverFault = 33,
    OverVoltage = 34,
    EncoderFault = 35,
    MotorNotConfigured = 36,
    PwmCycleOverrun = 37,
    OverTemperature = 38,
    StartOutsideLimit = 39,
    UnderVoltage = 40,
    ConfigChanged = 41,
    ThetaInvalid = 42,
    PositionInvalid = 43,
    DriverEnableFault = 44,
    StopPositionDeprecated = 45,
    TimingViolation = 46,
}

impl TryIntoBytes for Faults {
    fn try_into_1_byte(self, _scale: f32) -> Result<u8, RegisterError> {
        Ok(self as u8)
    }

    fn try_into_2_bytes(self, _scale: f32) -> Result<[u8; 2], RegisterError> {
        Ok((self as i16).to_le_bytes())
    }

    fn try_into_4_bytes(self, _scale: f32) -> Result<[u8; 4], RegisterError> {
        Ok((self as i32).to_le_bytes())
    }
    fn try_into_f32_bytes(self) -> Result<[u8; 4], RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

impl TryFromBytes for Faults {
    fn try_from_1_byte(byte: u8, _scale: f32) -> Result<Self, RegisterError> {
        Faults::from_u8(byte).ok_or(RegisterError::InvalidData)
    }
    fn try_from_2_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let value = u16::from_le_bytes([bytes[0], bytes[1]]);
        Faults::from_u16(value).ok_or(RegisterError::InvalidData)
    }
    fn try_from_4_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Faults::from_u32(value).ok_or(RegisterError::InvalidData)
    }
    fn try_from_f32_bytes(_bytes: &[u8]) -> Result<Self, RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

#[derive(Debug, Clone, Copy, AsBytes, FromPrimitive, PartialEq, Eq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum HomeStates {
    Relative = 0,
    Rotor = 1,
    Output = 2,
}

impl TryIntoBytes for HomeStates {
    fn try_into_1_byte(self, _scale: f32) -> Result<u8, RegisterError> {
        Ok(self as u8)
    }

    fn try_into_2_bytes(self, _scale: f32) -> Result<[u8; 2], RegisterError> {
        Ok((self as i16).to_le_bytes())
    }

    fn try_into_4_bytes(self, _scale: f32) -> Result<[u8; 4], RegisterError> {
        Ok((self as i32).to_le_bytes())
    }
    fn try_into_f32_bytes(self) -> Result<[u8; 4], RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

impl TryFromBytes for HomeStates {
    fn try_from_1_byte(byte: u8, _scale: f32) -> Result<Self, RegisterError> {
        HomeStates::from_u8(byte).ok_or(RegisterError::InvalidData)
    }
    fn try_from_2_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let value = u16::from_le_bytes([bytes[0], bytes[1]]);
        HomeStates::from_u16(value).ok_or(RegisterError::InvalidData)
    }
    fn try_from_4_bytes(bytes: &[u8], _scale: f32) -> Result<Self, RegisterError> {
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        HomeStates::from_u32(value).ok_or(RegisterError::InvalidData)
    }
    fn try_from_f32_bytes(_bytes: &[u8]) -> Result<Self, RegisterError> {
        Err(RegisterError::IntAsFloat)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_f32_register() {
        let position = Position::write(2.0);
        let data = position.as_bytes().unwrap();
        assert_eq!(data, vec![0, 0, 0, 64]);
        let from_data = Position::from_bytes(&data, Resolution::Float).unwrap();
        assert_eq!(from_data, Position::write(2.0));

        let data = Position::write_with_resolution(2.0, Resolution::Int8).as_bytes();
        assert!(data.is_err()); // OVERFLOW
        let data = Position::write_with_resolution(2.0, Resolution::Int16)
            .as_bytes()
            .unwrap();
        assert_eq!(data, 20000i16.to_le_bytes().to_vec());
        let data = Position::write_with_resolution(2.0, Resolution::Int32)
            .as_bytes()
            .unwrap();
        assert_eq!(data, 200000i32.to_le_bytes().to_vec());
        let data = Position::write_with_resolution(2.0, Resolution::Float)
            .as_bytes()
            .unwrap();
        assert_eq!(data, 2.0f32.to_le_bytes().to_vec());

        let position = Position::write(-2.0);
        let data = position.as_bytes().unwrap();
        assert_eq!(data, vec![0, 0, 0, 192]);
        let from_data = Position::from_bytes(&data, Resolution::Float).unwrap();
        assert_eq!(from_data, Position::write(-2.0));

        let data = Position::write_with_resolution(-2.0, Resolution::Int8).as_bytes();
        assert!(data.is_err()); // OVERFLOW
        let data = Position::write_with_resolution(-2.0, Resolution::Int16)
            .as_bytes()
            .unwrap();
        assert_eq!(data, (-20000i16).to_le_bytes().to_vec());
        let data = Position::write_with_resolution(-2.0, Resolution::Int32)
            .as_bytes()
            .unwrap();
        assert_eq!(data, (-200000i32).to_le_bytes().to_vec());
        let data = Position::write_with_resolution(-2.0, Resolution::Float)
            .as_bytes()
            .unwrap();
        assert_eq!(data, (-2.0f32).to_le_bytes().to_vec());
    }

    #[test]
    fn test_u8_register() {
        let data = Mode::write_with_resolution(Modes::Voltage, Resolution::Int8)
            .as_bytes()
            .unwrap();
        assert_eq!(data, vec![6]);
        let data = Mode::from_bytes(&data, Resolution::Int8).unwrap();
        assert_eq!(data, Mode::write(Modes::Voltage));
        let data = Mode::write_with_resolution(Modes::Voltage, Resolution::Int16)
            .as_bytes()
            .unwrap();
        assert_eq!(data, [6, 0].to_vec());
        let data = Mode::write_with_resolution(Modes::Voltage, Resolution::Int32)
            .as_bytes()
            .unwrap();
        assert_eq!(data, [6, 0, 0, 0].to_vec());
        let data = Mode::write_with_resolution(Modes::Voltage, Resolution::Float).as_bytes();
        assert!(data.is_err()); // IntAsFloat
    }

    #[test]
    fn test_i32_register() {
        let data = MillisecondCounter::write_with_resolution(1, Resolution::Int8)
            .as_bytes()
            .unwrap();
        assert_eq!(data, vec!(1));
        let data = MillisecondCounter::write_with_resolution(1, Resolution::Int16)
            .as_bytes()
            .unwrap();
        assert_eq!(data, vec!(1, 0));
        let data = MillisecondCounter::write_with_resolution(1, Resolution::Int32)
            .as_bytes()
            .unwrap();
        assert_eq!(data, vec!(1, 0, 0, 0));
        let data = MillisecondCounter::write_with_resolution(1, Resolution::Float).as_bytes();
        assert!(data.is_err());

        let data = MillisecondCounter::write_with_resolution(200, Resolution::Int8).as_bytes();
        assert!(data.is_err());
    }

    #[test]
    fn test_f32_nan() {
        let data = Position::write_with_resolution(f32::NAN, Resolution::Float)
            .as_bytes()
            .unwrap();
        assert_eq!(data, vec!(0, 0, 192, 127));
        assert!(Position::from_bytes(&data, Resolution::Float)
            .unwrap()
            .value
            .unwrap()
            .is_nan());

        let data = Position::write_with_resolution(f32::NAN, Resolution::Int8)
            .as_bytes()
            .unwrap();
        assert_eq!(data, vec!(i8::MIN as u8));
        assert!(Position::from_bytes(&data, Resolution::Int8)
            .unwrap()
            .value
            .unwrap()
            .is_nan());

        let data = Position::write_with_resolution(f32::NAN, Resolution::Int16)
            .as_bytes()
            .unwrap();
        assert_eq!(data, vec!(0, 128));
        assert!(Position::from_bytes(&data, Resolution::Int16)
            .unwrap()
            .value
            .unwrap()
            .is_nan());

        let data = Position::write_with_resolution(f32::NAN, Resolution::Int32)
            .as_bytes()
            .unwrap();
        assert_eq!(data, vec!(0, 0, 0, 128));
        assert!(Position::from_bytes(&data, Resolution::Int32)
            .unwrap()
            .value
            .unwrap()
            .is_nan());
    }

    #[test]
    fn get_data_from_bytes() {
        let reg = RegisterDataStruct {
            address: RegisterAddr::Position,
            resolution: Resolution::Float,
            data: Some([1, 0, 0, 0].into()),
        };
        let data = reg.as_reg::<Position>().unwrap();
        dbg!(&data);
    }
}
