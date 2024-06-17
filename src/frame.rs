//! [`Frame`]s are high-level structs used to format read registers and write registers with data into CAN-FD frames.
//!
//! [`FrameBuilder`] is provided to make creating [`Frame`]s easier.
//!
//! Many structs are provided to make creating [`Frame`]s easier.
//! For example, [`Position`] is used to create a frame to set the position of a motor.
//! These structs can be converted into a [`FrameBuilder`] using the `From` and `Into` traits.

use crate::protocol::{Frame, FrameBuilder};
use crate::registers::RegisterData;
use crate::{registers, Resolution};

/// Sets the mode to `registers::Modes::Stopped`.
#[derive(Debug, Default, Clone)]
pub struct Stop;

impl From<Stop> for FrameBuilder {
    fn from(_: Stop) -> FrameBuilder {
        Frame::builder().add([registers::Mode::write(registers::Modes::Stopped).into()])
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
    pub position: Option<registers::CommandPosition>,
    /// The `velocity` field is used to set the [`registers::CommandVelocity`] of the motor.
    pub velocity: Option<registers::CommandVelocity>,
    /// The `feedforward_torque` field is used to set the [`registers::CommandFeedforwardTorque`] of the motor.
    pub feedforward_torque: Option<registers::CommandFeedforwardTorque>,
    /// The `kp_scale` field is used to set the [`registers::CommandKpScale`] of the motor.
    pub kp_scale: Option<registers::CommandKpScale>,
    /// The `kd_scale` field is used to set the [`registers::CommandKdScale`] of the motor.
    pub kd_scale: Option<registers::CommandKdScale>,
    /// The `maximum_torque` field is used to set the [`registers::CommandPositionMaxTorque`] of the motor.
    pub maximum_torque: Option<registers::CommandPositionMaxTorque>,
    /// The `stop_position` field is used to set the [`registers::CommandStopPosition`] of the motor.
    pub stop_position: Option<registers::CommandStopPosition>,
    /// The `watchdog_timeout` field is used to set the [`registers::CommandTimeout`] of the motor.
    pub watchdog_timeout: Option<registers::CommandTimeout>,
    /// The `velocity_limit` field is used to set the [`registers::VelocityLimit`] of the motor.
    pub velocity_limit: Option<registers::VelocityLimit>,
    /// The `acceleration_limit` field is used to set the [`registers::AccelerationLimit`] of the motor.
    pub acceleration_limit: Option<registers::AccelerationLimit>,
    /// The `fixed_voltage_override` field is used to set the [`registers::FixedVoltage`] of the motor.
    pub fixed_voltage_override: Option<registers::FixedVoltage>,
    // todo: add query override
}

impl Position {
    /// Sets the [`registers::CommandPosition`] to `f32::NAN` to hold the current position.
    pub fn hold() -> Self {
        Self {
            position: Some(registers::CommandPosition::write(f32::NAN)),
            ..Self::default()
        }
    }

    /// Use a closure to config the position frame.
    pub fn configure<F: FnOnce(&mut Self)>(mut self, f: F) -> Self {
        f(&mut self);
        self
    }
}

impl IntoIterator for Position {
    type Item = registers::RegisterDataStruct;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Some(registers::Mode::write(registers::Modes::Position).into()),
            self.position.map(|p| p.into()),
            self.velocity.map(|v| v.into()),
            self.feedforward_torque.map(|f| f.into()),
            self.kp_scale.map(|k| k.into()),
            self.kd_scale.map(|k| k.into()),
            self.maximum_torque.map(|m| m.into()),
            self.stop_position.map(|s| s.into()),
            self.watchdog_timeout.map(|w| w.into()),
            self.velocity_limit.map(|v| v.into()),
            self.acceleration_limit.map(|a| a.into()),
            self.fixed_voltage_override.map(|f| f.into()),
        ]
            .into_iter()
            .flatten()
            .collect::<Vec<registers::RegisterDataStruct>>()
            .into_iter()
    }
}

/// Specify which query is merged into the frame being sent.
#[derive(Debug, Clone)]
pub enum QueryType {
    /// Sends the [`Controller`]s default query.
    Default,
    /// Sends the [`Controller`]s default query, merged with the provided [`FrameBuilder`].
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
    pub mode: Option<registers::Mode>,
    pub position: Option<registers::Position>,
    pub velocity: Option<registers::Velocity>,
    pub torque: Option<registers::Torque>,
    pub q_current: Option<registers::QCurrent>,
    pub d_current: Option<registers::DCurrent>,
    pub abs_position: Option<registers::AbsPosition>,
    pub motor_temperature: Option<registers::MotorTemperature>,
    pub trajectory_complete: Option<registers::TrajectoryComplete>,
    // rezero_state: Option<registers::RezeroState>,
    pub home_state: Option<registers::HomeState>,
    pub voltage: Option<registers::Voltage>,
    pub temperature: Option<registers::Temperature>,
    pub fault: Option<registers::Fault>,
    pub aux1_gpio: Option<registers::Aux1gpioStatus>,
    pub aux2_gpio: Option<registers::Aux1gpioStatus>,

    pub extra: Option<Vec<registers::RegisterDataStruct>>,
}

impl Query {
    /// Creates a new [`Query`] with the fields set with sensible defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Extends the default query with extra registers.
    pub fn new_with_extra<T>(extra: T) -> Self
        where
            T: IntoIterator<Item=registers::RegisterDataStruct>,
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

impl IntoIterator for Query {
    type Item = registers::RegisterDataStruct;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            self.mode.map(|m| m.into()),
            self.position.map(|p| p.into()),
            self.velocity.map(|v| v.into()),
            self.torque.map(|f| f.into()),
            self.q_current.map(|k| k.into()),
            self.d_current.map(|k| k.into()),
            self.abs_position.map(|m| m.into()),
            self.motor_temperature.map(|s| s.into()),
            self.trajectory_complete.map(|w| w.into()),
            self.home_state.map(|v| v.into()),
            self.voltage.map(|a| a.into()),
            self.temperature.map(|f| f.into()),
            self.fault.map(|f| f.into()),
            self.aux1_gpio.map(|f| f.into()),
            self.aux2_gpio.map(|f| f.into()),
        ]
            .into_iter()
            .flatten()
            .collect::<Vec<registers::RegisterDataStruct>>()
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use fdcanusb::FdCanUSB;

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

        let custom =
            Frame::builder().add([registers::Mode::write(registers::Modes::Position).into()]);
        let _ = c.query(1, QueryType::Custom(custom));

        let custom =
            Frame::builder().add([registers::Mode::write(registers::Modes::Position).into()]);
        let _ = c.query(1, QueryType::DefaultAnd(custom));
    }
}
