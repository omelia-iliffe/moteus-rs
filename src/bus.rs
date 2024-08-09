use crate::error::Error;
use crate::frame::QueryType;
use crate::protocol::{Frame, FrameBuilder, ResponseFrame};
use fdcanusb::{CanFdFrame, FdCanUSB};

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
    pub fn serial2(
        path: impl AsRef<std::path::Path>,
        serial_settings: impl fdcanusb::serial2::IntoSettings,
        disable_brs: bool,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            transport: FdCanUSB::open(path, serial_settings)?,
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
    /// ```rust
    /// # use moteus::frame::Query;
    /// # use moteus::registers::*;
    /// # use moteus::Controller;
    /// # fn main() -> Result<(), moteus::Error> {
    /// let qr = Query::new_with_extra([
    ///     ControlPosition::read().into(),
    ///     ControlVelocity::read().into(),
    ///     ControlTorque::read().into(),
    ///     ControlPositionError::read().into(),
    ///     ControlVelocityError::read().into(),
    ///     ControlTorqueError::read().into(),
    /// ]);
    /// let mut transport = fdcanusb::FdCanUSB::open("/dev/fdcanusb", fdcanusb::serial2::KeepSettings)?;
    /// transport.flush()?;
    /// let mut c = Controller::with_query(transport, false, qr);
    /// # Ok(())
    /// # }
    /// ```
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
        self.transfer_single_with_response(id, frame)
    }

    /// Send a single frame to the moteus. No response will be returned.
    /// Use [`Controller::send_with_query`] to get a response.
    pub fn send_no_response<F: Into<FrameBuilder>>(
        &mut self,
        id: u8,
        frame: F,
    ) -> Result<(), Error> {
        let frame = frame.into().build();
        self.transfer_single_no_response(id, frame)
    }

    /// Sends a single frame with a query to the moteus and returns a [`ResponseFrame`].
    ///
    /// The query frame can be set with [`QueryType`].
    /// Use [`QueryType::Default`] to use the default query frame.
    /// Use [`QueryType::DefaultAnd`] to merge the default query frame with a custom query frame.
    /// Use [`QueryType::Custom`] to use a custom query frame (without the default).
    pub fn send_with_query<F: Into<FrameBuilder>>(
        &mut self,
        id: u8,
        frame: F,
        query: QueryType,
    ) -> Result<ResponseFrame, Error> {
        let frame = match query {
            QueryType::Default => frame.into().merge(self.default_query.clone()).build(),
            QueryType::DefaultAnd(q_frame) => frame
                .into()
                .merge(self.default_query.clone())
                .merge(q_frame)
                .build(),
            QueryType::Custom(q_frame) => frame.into().merge(q_frame).build(),
        };
        self.transfer_single_with_response(id, frame)
    }

    fn transfer_single_no_response<F>(&mut self, id: u8, frame: F) -> Result<(), Error>
    where
        F: Into<Frame>,
    {
        let frame = frame.into();
        let arbitration_id = id as u16;
        let frame = CanFdFrame::new(
            arbitration_id,
            &frame.as_bytes().expect("Could not convert frame to bytes"),
        )?;
        let _ = self.transport.transfer_single(frame, false)?;
        Ok(())
    }
    fn transfer_single_with_response<F>(&mut self, id: u8, frame: F) -> Result<ResponseFrame, Error>
    where
        F: Into<Frame>,
    {
        let frame = frame.into();
        let arbitration_id = id as u16 | 0x8000;
        let frame = CanFdFrame::new(arbitration_id, &frame.as_bytes()?)?;
        let response = self.transport.transfer_single(frame, true)?;
        let response = response.ok_or(Error::NoResponse)?;
        Ok(response.try_into()?)
    }
}
