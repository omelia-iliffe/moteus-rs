
use zerocopy::AsBytes;
use num_derive::FromPrimitive;
#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq, AsBytes)]
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
    WriteError = 0x30,
    ReadError = 0x31,
    StreamClientData = 0x40,
    StreamServerData = 0x41,
    StreamClientPoll = 0x42,
    Nop = 0x50,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Data {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    F32(f32),
    None,
}
impl Data {
    pub fn is_none(&self) -> bool {
        matches!(self, Data::None)
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Data::Int8(value) => value.to_le_bytes().to_vec(),
            Data::Int16(value) => value.to_le_bytes().to_vec(),
            Data::Int32(value) => value.to_le_bytes().to_vec(),
            Data::F32(value) => value.to_le_bytes().to_vec(),
            Data::None => vec![],
        }
    }
}

#[derive(Debug)]
pub struct InvalidDataType;
impl From<i8> for Data {
    fn from(value: i8) -> Self {
        Data::Int8(value)
    }
}
impl From<i16> for Data {
    fn from(value: i16) -> Self {
        Data::Int16(value)
    }
}
impl From<i32> for Data {
    fn from(value: i32) -> Self {
        Data::Int32(value)
    }
}
impl From<f32> for Data {
    fn from(value: f32) -> Self {
        Data::F32(value)
    }
}
impl TryFrom<Data> for i8 {
    type Error = InvalidDataType;
    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Int8(value) => Ok(value),
            _ => Err(InvalidDataType),
        }
    }
}
impl TryFrom<Data> for i16 {
    type Error = InvalidDataType;
    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Int16(value) => Ok(value),
            _ => Err(InvalidDataType),
        }
    }
}
impl TryFrom<Data> for i32 {
    type Error = InvalidDataType;
    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Int32(value) => Ok(value),
            _ => Err(InvalidDataType),
        }
    }
}
impl TryFrom<Data> for f32 {
    type Error = InvalidDataType;
    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::F32(value) => Ok(value),
            _ => Err(InvalidDataType),
        }
    }
}

#[derive(Debug, Clone, Copy, AsBytes, FromPrimitive, PartialEq, Eq)]
#[repr(u16)]
pub enum Register {
    Mode = 0x000,
    Position = 0x001,
    Velocity = 0x002,
    Torque = 0x003,
    Qcurrent = 0x004,
    Dcurrent = 0x005,
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

    CommandQcurrent = 0x01c,
    CommandDcurrent = 0x01d,

    CommandPosition = 0x020,
    CommandVelocity = 0x021,
    CommandFeedforwardTorque = 0x022,
    CommandKpScale = 0x023,
    CommandKdScale = 0x024,
    CommandPositionMaxTorque = 0x025,
    CommandStopPosition = 0x026,
    CommandTimeout = 0x027,

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

#[derive(Debug, Clone, Copy, AsBytes, FromPrimitive)]
#[repr(u8)]
pub enum Mode {
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

#[derive(Debug, Clone, Copy, AsBytes, FromPrimitive)]
#[repr(u8)]
pub enum HomeState {
    Relative = 0,
    Rotor = 1,
    Output = 2,
}
