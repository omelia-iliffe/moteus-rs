use fdcanusb::{CanFdFrame, FdCanUSB};

use crate::frame::QueryType;
use crate::protocol::{Frame, FrameBuilder, FrameError, FrameParseError, ResponseFrame};

/// Errors that can occur when interacting with the Moteus.
#[derive(Debug)]
pub enum Error {
    /// IO errors occur when
    Io(std::io::Error),
    /// Frame errors occur when creating frames from an invalid combination of registers.
    Frame(FrameError),
    /// FrameParse errors occur when parsing frames from invalid bytes.
    FrameParse(FrameParseError),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(_f, "IO error: {}", e),
            Error::Frame(e) => write!(_f, "Frame error: {:?}", e),
            Error::FrameParse(e) => write!(_f, "Frame parse error: {:?}", e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<FrameError> for Error {
    fn from(e: FrameError) -> Self {
        Error::Frame(e)
    }
}

impl From<FrameParseError> for Error {
    fn from(e: FrameParseError) -> Self {
        Error::FrameParse(e)
    }
}

/// The main struct for interacting with the Moteus.
pub struct Controller<T>
    where
        T: std::io::Write + std::io::Read,
{
    transport: FdCanUSB<T>,
    default_query: FrameBuilder,
    /// Disable BRS (Bit Rate Switch) in the CAN FD frames. Useful if your CAN network is unable to perform.
    pub disable_brs: bool,
}

#[cfg(feature = "serial2")]
impl Controller<fdcanusb::serial2::SerialPort> {
    /// Create a new [`Controller`] instance with a given transport.
    ///
    /// Currently, the transport is limited to [`FdCanUSB`].
    ///
    /// ```rust
    /// #[cfg(feature = "serial2")]
    /// # fn main() -> std::io::Result<()> {
    /// moteus::Controller::new(moteus::FdCanUSB::open("/dev/fdcanusb", moteus::serial2::KeepSettings)?, false);
    /// # Ok(())
    /// }
    /// ```
    pub fn serial2(path: impl AsRef<std::path::Path>, serial_settings: impl moteus::serial2::IntoSettings, disable_brs: bool) -> Result<Self, std::io::Error> {
        Ok(Self {
            transport: FdCanUSB::open(path, fdcanusb::serial2::KeepSettings)?,
            default_query: crate::frame::Query::default().into(),
            disable_brs,
        })
    }
}

impl<T> Controller<T>
    where
        T: std::io::Write + std::io::Read,
{
    /// Create a new [`Controller`] instance with a given transport.
    ///
    /// Currently, the transport is limited to [`FdCanUSB`].
    ///
    /// ```rust
    /// #[cfg(feature = "serial2")]
    /// # fn main() -> std::io::Result<()> {
    /// moteus::Controller::new(moteus::FdCanUSB::open("/dev/fdcanusb", moteus::serial2::KeepSettings)?, false);
    /// # Ok(())
    /// }
    /// ```
    pub fn new(transport: FdCanUSB<T>, disable_brs: bool) -> Self {
        Self {
            transport,
            default_query: crate::frame::Query::default().into(),
            disable_brs,
        }
    }
    /// Creates a new [`Controller`] instance with a custom default query.
    ///
    /// todo: add example
    pub fn with_query<F>(transport: FdCanUSB<T>, disable_brs: bool, default_query: F) -> Self
        where
            F: Into<FrameBuilder>,
    {
        Controller {
            transport,
            default_query: default_query.into(),
            disable_brs,
        }
    }

    /// Sends a single query frame to the moteus and returns a [`ResponseFrame`].
    ///
    /// The query frame can be set with [`QueryType`].
    /// Use [`QueryType::Default`] to use the default query frame.
    /// Use [`QueryType::DefaultAnd`] to merge the default query frame with a custom query frame.
    /// Use [`QueryType::Custom`] to use a custom query frame (without the default).
    pub fn query(&mut self, id: u8, query: QueryType) -> Result<ResponseFrame, Error> {
        let frame = match query {
            QueryType::Default => self.default_query.clone().build(),
            QueryType::DefaultAnd(q_frame) => self.default_query.clone().merge(q_frame).build(),
            QueryType::Custom(q_frame) => q_frame.build(),
        };
        self.transfer_single(id, frame, true).map(|r| r.expect("Expected response!"))
    }

    /// Send a single frame to the moteus. If `query` is `Some`, a [`ResponseFrame`] will be returned.
    /// If `query` is `None`, no response is expected and `Ok(None)` will be returned.
    /// The query frame can be set with [`QueryType`].
    /// Use [`QueryType::Default`] to use the default query frame.
    /// Use [`QueryType::DefaultAnd`] to merge the default query frame with a custom query frame.
    /// Use [`QueryType::Custom`] to use a custom query frame (without the default).
    pub fn send<F: Into<FrameBuilder>>(
        &mut self,
        id: u8,
        frame: F,
        query: Option<QueryType>,
    ) -> Result<Option<ResponseFrame>, Error> {
        let expect_response = query.is_some();
        let frame = match query {
            None => frame.into().build(),
            Some(QueryType::Default) => frame.into().merge(self.default_query.clone()).build(),
            Some(QueryType::DefaultAnd(q_frame)) => frame.into().merge(self.default_query.clone()).merge(q_frame).build(),
            Some(QueryType::Custom(q_frame)) => frame.into().merge(q_frame).build(),
        };
        self.transfer_single(id, frame, expect_response)
    }

    fn transfer_single<F>(
        &mut self,
        id: u8,
        frame: F,
        expect_response: bool,
    ) -> Result<Option<ResponseFrame>, Error>
        where
            F: Into<Frame>,
    {
        let frame = frame.into();
        let arbitration_id = {
            match expect_response {
                false => id as u16,
                true => id as u16 | 0x8000,
            }
        };
        let frame = CanFdFrame::new(arbitration_id, &frame.as_bytes().expect("Could not convert frame to bytes"));
        let response = self.transport.transfer_single(frame, expect_response)?;
        if !expect_response {
            return Ok(None);
        }
        let response = response.ok_or(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No response",
        )))?;
        Ok(Some(response.try_into()?))
    }
}
