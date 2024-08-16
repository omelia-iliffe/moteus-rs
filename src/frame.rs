//! This module contain structs which can be used to build common frames, such as [`Stop`] and [`Position`]
//! each impl Into<[`FrameBuilder`]> and can be passed into functions such as [`crate::Controller::send_with_query`].

use crate::protocol::{Frame, FrameBuilder};
use crate::registers::{Read, Readable, Write, Writeable};
use crate::{registers, Error, Resolution};

/// Sets the mode to `registers::Modes::Stopped`.
#[derive(Debug, Default, Clone)]
pub struct Stop;

impl From<Stop> for FrameBuilder {
    fn from(_: Stop) -> FrameBuilder {
        let mut builder = Frame::builder();
        builder.add(registers::Mode::write(registers::Modes::Stopped).unwrap());
        builder
    }
}

/// Sets the mode to `registers::Modes::Position`.
///
/// Each field is optional, and if a field is `None`, the corresponding register is omitted from the frame.
///
/// Additionally, some associated methods are provided. See:
///  - [`Position::hold`]
#[derive(Debug, Default, Clone)]
pub struct Position {
    /// The `position` field is used to set the [`registers::CommandPosition`] of the motor.
    pub position: Option<Write<registers::CommandPosition>>,
    /// The `velocity` field is used to set the [`registers::CommandVelocity`] of the motor.
    pub velocity: Option<Write<registers::CommandVelocity>>,
    /// The `feedforward_torque` field is used to set the [`registers::CommandFeedforwardTorque`] of the motor.
    pub feedforward_torque: Option<Write<registers::CommandFeedforwardTorque>>,
    /// The `kp_scale` field is used to set the [`registers::CommandKpScale`] of the motor.
    pub kp_scale: Option<Write<registers::CommandKpScale>>,
    /// The `kd_scale` field is used to set the [`registers::CommandKdScale`] of the motor.
    pub kd_scale: Option<Write<registers::CommandKdScale>>,
    /// The `maximum_torque` field is used to set the [`registers::CommandPositionMaxTorque`] of the motor.
    pub maximum_torque: Option<Write<registers::CommandPositionMaxTorque>>,
    /// The `stop_position` field is used to set the [`registers::CommandStopPosition`] of the motor.
    pub stop_position: Option<Write<registers::CommandStopPosition>>,
    /// The `watchdog_timeout` field is used to set the [`registers::CommandTimeout`] of the motor.
    pub watchdog_timeout: Option<Write<registers::CommandTimeout>>,
    /// The `velocity_limit` field is used to set the [`registers::VelocityLimit`] of the motor.
    pub velocity_limit: Option<Write<registers::VelocityLimit>>,
    /// The `acceleration_limit` field is used to set the [`registers::AccelerationLimit`] of the motor.
    pub acceleration_limit: Option<Write<registers::AccelerationLimit>>,
    /// The `fixed_voltage_override` field is used to set the [`registers::FixedVoltage`] of the motor.
    pub fixed_voltage_override: Option<Write<registers::FixedVoltage>>,
}

impl Position {
    /// Sets the [`registers::CommandPosition`] to `f32::NAN` to hold the current position.
    pub fn hold() -> Self {
        Self {
            position: Some(registers::CommandPosition::write(f32::NAN).unwrap()),
            ..Self::default()
        }
    }

    /// Use a closure to config the position frame.
    pub fn configure<F>(mut self, f: F) -> Result<Self, Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Error>,
    {
        f(&mut self)?;
        Ok(self)
    }
}

impl From<Position> for FrameBuilder {
    fn from(position: Position) -> Self {
        let mut builder = Frame::builder();
        builder.add(registers::Mode::write(registers::Modes::Position).unwrap());
        if let Some(p) = position.position {
            builder.add(p);
        }
        if let Some(v) = position.velocity {
            builder.add(v);
        }
        if let Some(t) = position.feedforward_torque {
            builder.add(t);
        }
        if let Some(kp) = position.kp_scale {
            builder.add(kp);
        }
        if let Some(kd) = position.kd_scale {
            builder.add(kd);
        }
        if let Some(t) = position.maximum_torque {
            builder.add(t);
        }
        if let Some(s) = position.stop_position {
            builder.add(s);
        }
        if let Some(w) = position.watchdog_timeout {
            builder.add(w);
        }
        if let Some(v) = position.velocity_limit {
            builder.add(v);
        }
        if let Some(a) = position.acceleration_limit {
            builder.add(a);
        }
        if let Some(f) = position.fixed_voltage_override {
            builder.add(f);
        }
        builder
    }
}

/// Specify which query is merged into the frame being sent.
#[derive(Debug, Clone)]
pub enum QueryType {
    /// Sends the [`crate::Controller`]s default query.
    Default,
    /// Sends the [`crate::Controller`]s default query, merged with the provided [`FrameBuilder`].
    DefaultAnd(FrameBuilder),
    /// Sends the provided [`FrameBuilder`].
    Custom(FrameBuilder),
}

/// A query is a collection of registers to be read from the motor.
/// The fields are some useful registers that are commonly queried, but any register can be added to the `extra` field.
///
/// The default query is:
/// - `Mode` with resolution `Resolution::Int8`
/// - `Position` with resolution `Resolution::Float`
/// - `Velocity` with resolution `Resolution::Float`
/// - `Torque` with resolution `Resolution::Float`
/// - `Voltage` with resolution `Resolution::Int8`
/// - `Temperature` with resolution `Resolution::Int8`
/// - `Fault` with resolution `Resolution::Int8`
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct Query {
    pub mode: Option<Read<registers::Mode>>,
    pub position: Option<Read<registers::Position>>,
    pub velocity: Option<Read<registers::Velocity>>,
    pub torque: Option<Read<registers::Torque>>,
    pub q_current: Option<Read<registers::QCurrent>>,
    pub d_current: Option<Read<registers::DCurrent>>,
    pub abs_position: Option<Read<registers::AbsPosition>>,
    pub motor_temperature: Option<Read<registers::MotorTemperature>>,
    pub trajectory_complete: Option<Read<registers::TrajectoryComplete>>,
    // rezero_state: Option<registers::RezeroState>,
    pub home_state: Option<Read<registers::HomeState>>,
    pub voltage: Option<Read<registers::Voltage>>,
    pub temperature: Option<Read<registers::Temperature>>,
    pub fault: Option<Read<registers::Fault>>,
    pub aux1_gpio: Option<Read<registers::Aux1gpioStatus>>,
    pub aux2_gpio: Option<Read<registers::Aux1gpioStatus>>,

    pub extra: Option<Vec<registers::RegisterData>>,
}

impl Query {
    /// Creates a new [`Query`] with the fields set with sensible defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Extends the default query with extra registers.
    pub fn new_with_extra<T>(extra: T) -> Self
    where
        T: IntoIterator<Item = registers::RegisterData>,
    {
        Self {
            extra: Some(extra.into_iter().collect::<Vec<_>>()),
            ..Self::default()
        }
    }
}

impl Default for Query {
    fn default() -> Self {
        Self {
            mode: Some(registers::Mode::read_with_resolution(Resolution::Int8)),
            position: Some(registers::Position::read_with_resolution(Resolution::Float)),
            velocity: Some(registers::Velocity::read_with_resolution(Resolution::Float)),
            torque: Some(registers::Torque::read_with_resolution(Resolution::Float)),
            q_current: None,
            d_current: None,
            abs_position: None,
            motor_temperature: None,
            trajectory_complete: None,
            // rezero_state: None,
            home_state: None,
            voltage: Some(registers::Voltage::read_with_resolution(Resolution::Int8)),
            temperature: Some(registers::Temperature::read_with_resolution(
                Resolution::Int8,
            )),
            fault: Some(registers::Fault::read_with_resolution(Resolution::Int8)),
            aux1_gpio: None,
            aux2_gpio: None,
            extra: None,
        }
    }
}

impl From<Query> for FrameBuilder {
    fn from(query: Query) -> Self {
        let mut builder = Frame::builder();
        if let Some(m) = query.mode {
            builder.add(m);
        }
        if let Some(p) = query.position {
            builder.add(p);
        }
        if let Some(v) = query.velocity {
            builder.add(v);
        }
        if let Some(t) = query.torque {
            builder.add(t);
        }
        if let Some(q) = query.q_current {
            builder.add(q);
        }
        if let Some(d) = query.d_current {
            builder.add(d);
        }
        if let Some(a) = query.abs_position {
            builder.add(a);
        }
        if let Some(m) = query.motor_temperature {
            builder.add(m);
        }
        if let Some(t) = query.trajectory_complete {
            builder.add(t);
        }
        // if let Some(r) = query.rezero_state {
        //     builder.add(r);
        // }
        if let Some(h) = query.home_state {
            builder.add(h);
        }
        if let Some(v) = query.voltage {
            builder.add(v);
        }
        if let Some(t) = query.temperature {
            builder.add(t);
        }
        if let Some(f) = query.fault {
            builder.add(f);
        }
        if let Some(a) = query.aux1_gpio {
            builder.add(a);
        }
        if let Some(a) = query.aux2_gpio {
            builder.add(a);
        }
        if let Some(extra) = query.extra {
            for e in extra {
                builder.add(e);
            }
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use fdcanusb::{CanFdFrame, FdCanUSB, FdCanUSBFrame};

    use super::*;

    /// Will fail unless a motor is connected with id 1.
    #[test]
    fn test_query() {
        let mut c = crate::Controller::new(
            FdCanUSB::open("/dev/fdcanusb", fdcanusb::serial2::KeepSettings)
                .expect("Couldn't open fdcanusb at /dev/fdcanusb"),
            false,
        );
        let _ = c.query(1, QueryType::Default);

        let mut custom = Frame::builder();
        custom.add(registers::Mode::write(registers::Modes::Position).unwrap());
        let _ = c.query(1, QueryType::Custom(custom));

        let mut custom = Frame::builder();
        custom.add(registers::Mode::write(registers::Modes::Position).unwrap());
        let _ = c.query(1, QueryType::DefaultAnd(custom));
    }

    #[test]
    fn test_query_parse() {


        let recv: FdCanUSBFrame = "rcv 8001 01000A0E20000000BF000000000E2800000041000040401100130D1F011C0638505050\n".into();
        let frame = CanFdFrame::try_from(recv).unwrap();
        let frame: crate::ResponseFrame = frame.try_into().unwrap();
        dbg!(frame);

    }
}
