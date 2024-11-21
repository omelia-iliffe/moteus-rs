use crate::error::Error;
use crate::frame::QueryType;
use crate::protocol::{Frame, FrameBuilder, ResponseFrame};
use crate::FrameParseError;
use fdcanusb::CanFdFrame;

/// The main struct for interacting with the Moteus.
pub struct Controller<T> {
    transport: T,
    default_query: FrameBuilder,
    /// Disable BRS (Bit Rate Switch) in the CAN FD frames. Useful if your CAN network is unable to perform.
    pub disable_brs: bool,
}

#[cfg(feature = "fdcanusb")]
impl Controller<fdcanusb::FdCanUSB<fdcanusb::serial2::SerialPort>> {
    /// Create a new [`Controller`] instance with a given transport.
    ///
    /// Currently, the transport is limited to [`FdCanUSB`].
    ///
    /// ```rust
    /// # use fdcanusb::{FdCanUSB, serial2};
    /// # fn main() -> std::io::Result<()> {
    /// moteus::Controller::new(FdCanUSB::open("/dev/fdcanusb", serial2::KeepSettings)?, false);
    /// # Ok(())
    /// }
    /// ```
    pub fn fdcanusb(
        path: impl AsRef<std::path::Path>,
        serial_settings: impl fdcanusb::serial2::IntoSettings,
        disable_brs: bool,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            transport: fdcanusb::FdCanUSB::open(path, serial_settings)?,
            default_query: crate::frame::Query::default().into(),
            disable_brs,
        })
    }
}

impl<T, F> Controller<T>
where
    T: crate::transport::Transport<Frame = F>,
    F: From<CanFdFrame> + TryInto<ResponseFrame, Error = FrameParseError>,
{
    /// Create a new [`Controller`] instance with a given transport.
    ///
    /// Currently, the transport is limited to [`FdCanUSB`].
    ///
    /// ```rust
    /// # fn main() -> std::io::Result<()> {
    /// moteus::Controller::new(moteus::FdCanUSB::open("/dev/fdcanusb", moteus::serial2::KeepSettings)?, false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(transport: T, disable_brs: bool) -> Self {
        Self {
            transport,
            default_query: crate::frame::Query::default().into(),
            disable_brs,
        }
    }
    /// Creates a new [`Controller`] instance with a custom default query.
    ///
    /// ```rust
    /// # use moteus::frame::Query;
    /// # use moteus::registers::*;
    /// # use moteus::Controller;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    pub fn with_query(
        transport: T,
        disable_brs: bool,
        default_query: impl Into<FrameBuilder>,
    ) -> Self {
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
    pub fn query(&mut self, id: u8, query: QueryType) -> Result<ResponseFrame, Error<T::Error>> {
        let frame = match query {
            QueryType::Default => self.default_query.clone().build(),
            QueryType::DefaultAnd(q_frame) => self.default_query.clone().merge(q_frame).build(),
            QueryType::Custom(q_frame) => q_frame.build(),
        };
        self.transfer_single_with_response(id, frame)
    }

    /// Send a single frame to the moteus. No response will be returned.
    /// Use [`Controller::send_with_query`] to get a response.
    pub fn send_no_response(
        &mut self,
        id: u8,
        frame: impl Into<FrameBuilder>,
    ) -> Result<(), Error<T::Error>> {
        let frame = frame.into().build();
        self.transfer_single_no_response(id, frame)
    }

    /// Sends a single frame with a query to the moteus and returns a [`ResponseFrame`].
    ///
    /// The query frame can be set with [`QueryType`].
    /// Use [`QueryType::Default`] to use the default query frame.
    /// Use [`QueryType::DefaultAnd`] to merge the default query frame with a custom query frame.
    /// Use [`QueryType::Custom`] to use a custom query frame (without the default).
    pub fn send_with_query(
        &mut self,
        id: u8,
        frame: impl Into<FrameBuilder>,
        query: QueryType,
    ) -> Result<ResponseFrame, Error<T::Error>> {
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

    fn transfer_single_no_response(
        &mut self,
        id: u8,
        frame: impl Into<Frame>,
    ) -> Result<(), Error<T::Error>> {
        let frame = frame.into();
        let arbitration_id = id as u16;
        let frame = CanFdFrame {
            arbitration_id,
            data: frame.as_bytes()?,
            brs: Some(!self.disable_brs),
            ..Default::default()
        };
        self.transport.transmit(frame.into())?;
        Ok(())
    }
    fn transfer_single_with_response(
        &mut self,
        id: u8,
        frame: impl Into<Frame>,
    ) -> Result<ResponseFrame, Error<T::Error>> {
        let frame = frame.into();
        let arbitration_id = id as u16 | 0x8000;
        let frame = CanFdFrame {
            arbitration_id,
            data: frame.as_bytes()?,
            brs: Some(!self.disable_brs),
            ..Default::default()
        };
        self.transport.transmit(frame.into())?;
        let response = self.transport.receive()?;
        Ok(response.try_into()?)
    }
}
