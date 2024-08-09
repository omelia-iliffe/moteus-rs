use thiserror::Error;

/// Errors that can occur when interacting with the Moteus.
#[derive(Error, Debug)]
pub enum Error {
    /// IO errors occur when flushing the fdcanusb.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Transfer errors occur when reading or writing frames to the fdcanusb.
    #[error(transparent)]
    TransportError(#[from] fdcanusb::TransferError),
    /// Data overflow errors occur when the data is > 64 bytes.
    /// 64 bytes is the max frame length in the CAN FD protocol.
    #[error("data overflow error: {0}")]
    InvalidFrameLength(#[from] fdcanusb::InvalidFrameLength),
    /// Frame errors occur when creating frames from an invalid combination of registers.
    #[error("frame error: {0}")]
    Frame(#[from] FrameError),
    /// FrameParse errors occur when parsing frames from invalid bytes.
    #[error("frame parse error: {0}")]
    FrameParse(#[from] FrameParseError),
    /// No response was received.
    #[error("no response")]
    NoResponse,
}

/// Errors that can occur when creating frames from multiple subframes.
#[derive(Error, Debug)]
pub enum FrameError {
    /// The registers are not sequential. The subframe needs to be spilt into smaller sequential subframes.
    #[error("non-sequential registers")]
    NonSequentialRegisters,
    /// The subframe is empty. Subframes must contain at least one register to be valid
    #[error("Empty subframe")]
    EmptySubFrame,
    /// Frames can either contain registers to read (indicated with no data) or write registers with data. subframes cannot be mixed.
    #[error("mixed read and write registers")]
    MixedReadWrites,
    //
    // #[error("register error: {0}")]
    // RegisterError(#[from] RegisterError),
}

/// Subframe parsing errors occur when a sequence of bytes is parsed into a subframe.
#[derive(Error, Debug)]
pub enum FrameParseError {
    /// The subframe register address is invalid. valid addresses as defined in the [`crate::registers::FrameRegisters`] enum.
    #[error("invalid subframe register address: {0}")]
    InvalidFrameRegister(u8),
    // /// The subframe length is invalid. The length must be 8 bytes.
    // #[error("invalid frame length")]
    // SubFrameLength,
    // #[error("invalid register address")]
    // Register,
    /// The subframe register is not supported. When parsing frames, only a subset of the registers are supported.
    /// Supported registers are ReadRegisters, WriteRegisters, and NOP.
    // TODO: add support for remaining registers
    #[error("unsupported subframe register: {0:?}")]
    UnsupportedSubframeRegister(crate::registers::FrameRegisters),
    /// Subframes are collections of registers. Errors can occur when parsing each register.
    #[error("error parsing data into register: {0}")]
    RegisterError(#[from] RegisterError),
}

/// Errors that can occur when writing and/or parsing registers
#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum RegisterError {
    /// Returned when the value is too large to fit in the register
    #[error("value too large")]
    Overflow,
    /// Returned when data is tried into a type that is not valid.
    #[error("invalid data")]
    InvalidData,
    /// Returned when the parsed address of a register is invalid. All valid addresses are defined in the [`crate::registers::RegisterAddr`] enum
    #[error("invalid address")]
    InvalidAddress,
    /// Returned when a float is tried to be written to a register that only accepts integers
    #[error("float as int")]
    IntAsFloat,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    /// Returned when there is no mapping for the register
    #[error("register has no mapping")]
    NoMapping,
    /// Returned when writing is attempted with a register instance that doesn't have any data.
    #[error("cannot write register with no data")]
    NoData,
}
