
use zerocopy::AsBytes;
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum FrameRegisters {
    WRITE_INT8 = 0x00,
    WRITE_INT16 = 0x04,
    WRITE_INT32 = 0x08,
    WRITE_F32 = 0x0c,
    // READ_BASE = 0x10,
    READ_INT8 = 0x10,
    READ_INT16 = 0x14,
    READ_INT32 = 0x18,
    READ_F32 = 0x1c,
    REPLY_BASE = 0x20,
    WRITE_ERROR = 0x30,
    READ_ERROR = 0x31,
    STREAM_CLIENT_DATA = 0x40,
    STREAM_SERVER_DATA = 0x41,
    STREAM_CLIENT_POLL = 0x42,
    NOP = 0x50,
}

#[derive(Debug, Clone, Copy, AsBytes)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum Register {
    kMode = 0x000,
    kPosition = 0x001,
    kVelocity = 0x002,
    kTorque = 0x003,
    kQCurrent = 0x004,
    kDCurrent = 0x005,
    kAbsPosition = 0x006,

    kMotorTemperature = 0x00a,
    kTrajectoryComplete = 0x00b,
    kHomeState = 0x00c,
    kVoltage = 0x00d,
    kTemperature = 0x00e,
    kFault = 0x00f,

    kPwmPhaseA = 0x010,
    kPwmPhaseB = 0x011,
    kPwmPhaseC = 0x012,

    kVoltagePhaseA = 0x014,
    kVoltagePhaseB = 0x015,
    kVoltagePhaseC = 0x016,

    kVFocTheta = 0x018,
    kVFocVoltage = 0x019,
    kVoltageDqD = 0x01a,
    kVoltageDqQ = 0x01b,

    kCommandQCurrent = 0x01c,
    kCommandDCurrent = 0x01d,

    kCommandPosition = 0x020,
    kCommandVelocity = 0x021,
    kCommandFeedforwardTorque = 0x022,
    kCommandKpScale = 0x023,
    kCommandKdScale = 0x024,
    kCommandPositionMaxTorque = 0x025,
    kCommandStopPosition = 0x026,
    kCommandTimeout = 0x027,

    kPositionKp = 0x030,
    kPositionKi = 0x031,
    kPositionKd = 0x032,
    kPositionFeedforward = 0x033,
    kPositionCommand = 0x034,

    kControlPosition = 0x038,
    kControlVelocity = 0x039,
    kControlTorque = 0x03a,
    kControlPositionError = 0x03b,
    kControlVelocityError = 0x03c,
    kControlTorqueError = 0x03d,

    kCommandStayWithinLowerBound = 0x040,
    kCommandStayWithinUpperBound = 0x041,
    kCommandStayWithinFeedforwardTorque = 0x042,
    kCommandStayWithinKpScale = 0x043,
    kCommandStayWithinKdScale = 0x044,
    kCommandStayWithinPositionMaxTorque = 0x045,
    kCommandStayWithinTimeout = 0x046,

    kEncoder0Position = 0x050,
    kEncoder0Velocity = 0x051,
    kEncoder1Position = 0x052,
    kEncoder1Velocity = 0x053,
    kEncoder2Position = 0x054,
    kEncoder2Velocity = 0x055,

    kEncoderValidity = 0x058,

    kAux1GpioCommand = 0x05c,
    kAux2GpioCommand = 0x05d,
    kAux1GpioStatus = 0x05e,
    kAux2GpioStatus = 0x05f,

    kAux1AnalogIn1 = 0x060,
    kAux1AnalogIn2 = 0x061,
    kAux1AnalogIn3 = 0x062,
    kAux1AnalogIn4 = 0x063,
    kAux1AnalogIn5 = 0x064,

    kAux2AnalogIn1 = 0x068,
    kAux2AnalogIn2 = 0x069,
    kAux2AnalogIn3 = 0x06a,
    kAux2AnalogIn4 = 0x06b,
    kAux2AnalogIn5 = 0x06c,

    kMillisecondCounter = 0x070,
    kClockTrim = 0x071,

    kRegisterMapVersion = 0x102,
    kSerialNumber = 0x120,
    // kSerialNumber1 = 0x120,
    // kSerialNumber2 = 0x121,
    // kSerialNumber3 = 0x122,
    kRezero = 0x130,
    // kSetOutputNearest = 0x130,
    kSetOutputExact = 0x131,
    kRequireReindex = 0x132,

    kDriverFault1 = 0x140,
    kDriverFault2 = 0x141,
}

#[derive(Debug, Clone, Copy, AsBytes)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum Mode {
    kStopped = 0,
    kFault = 1,
    kEnabling = 2,
    kCalibrating = 3,
    kCalibrationComplete = 4,
    kPwm = 5,
    kVoltage = 6,
    kVoltageFoc = 7,
    kVoltageDq = 8,
    kCurrent = 9,
    kPosition = 10,
    kPositionTimeout = 11,
    kZeroVelocity = 12,
    kStayWithin = 13,
    kMeasureInd = 14,
    kBrake = 15,
}

#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum HomeState {
    kRelative = 0,
    kRotor = 1,
    kOutput = 2,
}
