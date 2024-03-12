use fdcanusb::{CanFdFrame, FdCanUSB};
use serial2::SerialPort;

use crate::frame::QueryType;
use crate::protocol::{Frame, FrameBuilder, FrameError, FrameParseError, ResponseFrame};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Frame(FrameError),
    FrameParse(FrameParseError),
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

pub struct Controller {
    pub transport: FdCanUSB<SerialPort>,
    default_query: FrameBuilder,
}

impl Default for Controller {
    fn default() -> Self {
        Controller {
            transport: FdCanUSB::new("/dev/fdcanusb", 115200),
            default_query: crate::frame::Query::default().into(),
        }
    }
}

impl Controller {
    pub fn with_query<F>(default_query: F) -> Self
    where
        F: Into<FrameBuilder>,
    {
        Controller {
            default_query: default_query.into(),
            ..Self::default()
        }
    }

    pub fn send<F: Into<FrameBuilder>>(
        &mut self,
        id: u8,
        frame: F,
        query: QueryType<F>,
    ) -> Result<Option<ResponseFrame>> {
        let expect_response = query.expect_repsonse();
        let frame = match query {
            QueryType::None => frame.into().build(),
            QueryType::Default => frame.into().merge(self.default_query.clone()).build(),
            QueryType::Custom(q_frame) => frame.into().merge(q_frame.into()).build(),
        };
        self.transfer_single(id, frame, expect_response)
    }
    fn transfer_single<F>(
        &mut self,
        id: u8,
        frame: F,
        expect_response: bool,
    ) -> Result<Option<ResponseFrame>>
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
        let frame = CanFdFrame::new(arbitration_id, &frame.as_bytes().unwrap());
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
