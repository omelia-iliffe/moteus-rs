use crate::protocol::{Frame, FrameBuilder};
use crate::registers::RegisterData;
use crate::{registers, Resolution};

#[derive(Debug, Default, Clone)]
pub struct Stop;

impl From<Stop> for FrameBuilder {
    fn from(_: Stop) -> FrameBuilder {
        Frame::builder().add([registers::Mode::write(registers::Modes::Stopped).into()])
    }
}

#[derive(Debug, Default, Clone)]
pub struct Position {
    pub position: Option<registers::CommandPosition>,
    pub velocity: Option<registers::CommandVelocity>,
    pub feedforward_torque: Option<registers::CommandFeedforwardTorque>,
    pub kp_scale: Option<registers::CommandKpScale>,
    pub kd_scale: Option<registers::CommandKdScale>,
    pub maximum_torque: Option<registers::CommandPositionMaxTorque>,
    pub stop_position: Option<registers::CommandStopPosition>,
    pub watchdog_timeout: Option<registers::CommandTimeout>,
    pub velocity_limit: Option<registers::VelocityLimit>,
    pub acceleration_limit: Option<registers::AccelerationLimit>,
    pub fixed_voltage_override: Option<registers::FixedVoltage>,
    // todo: add query override
}

impl Position {
    pub fn hold() -> Self {
        Self {
            position: Some(registers::CommandPosition::write(f32::NAN)),
            ..Self::default()
        }
    }
}

impl From<Position> for FrameBuilder {
    fn from(value: Position) -> Self {
        let mut fb = Frame::builder();
        fb = fb.add_single(registers::Mode::write(registers::Modes::Position).into());

        if let Some(position) = value.position {
            fb = fb.add_single(position.into());
        }
        if let Some(velocity) = value.velocity {
            fb = fb.add_single(velocity.into());
        }
        if let Some(feedforward_torque) = value.feedforward_torque {
            fb = fb.add_single(feedforward_torque.into());
        }
        if let Some(kp_scale) = value.kp_scale {
            fb = fb.add_single(kp_scale.into());
        }
        if let Some(kd_scale) = value.kd_scale {
            fb = fb.add_single(kd_scale.into());
        }
        if let Some(maximum_torque) = value.maximum_torque {
            fb = fb.add_single(maximum_torque.into());
        }
        if let Some(stop_position) = value.stop_position {
            fb = fb.add_single(stop_position.into());
        }
        if let Some(watchdog_timeout) = value.watchdog_timeout {
            fb = fb.add_single(watchdog_timeout.into());
        }
        if let Some(velocity_limit) = value.velocity_limit {
            fb = fb.add_single(velocity_limit.into());
        }
        if let Some(acceleration_limit) = value.acceleration_limit {
            fb = fb.add_single(acceleration_limit.into());
        }
        if let Some(fixed_voltage_override) = value.fixed_voltage_override {
            fb = fb.add_single(fixed_voltage_override.into());
        }
        fb
    }
}

#[derive(Debug, Clone)]
pub enum QueryType {
    None,
    Default,
    Custom(FrameBuilder),
}

impl QueryType {
    pub fn expect_repsonse(&self) -> bool {
        matches!(self, QueryType::Default | QueryType::Custom(_))
    }
}

impl From<FrameBuilder> for QueryType {
    fn from(fb: FrameBuilder) -> Self {
        QueryType::Custom(fb)
    }
}

#[derive(Debug, Clone)]
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
    fn from(query: Query) -> FrameBuilder {
        let mut fb = Frame::builder();

        if let Some(mode) = query.mode {
            fb = fb.add_single(mode.into());
        }
        if let Some(position) = query.position {
            fb = fb.add_single(position.into());
        }
        if let Some(velocity) = query.velocity {
            fb = fb.add_single(velocity.into());
        }
        if let Some(torque) = query.torque {
            fb = fb.add_single(torque.into());
        }
        if let Some(q_current) = query.q_current {
            fb = fb.add_single(q_current.into());
        }
        if let Some(d_current) = query.d_current {
            fb = fb.add_single(d_current.into());
        }
        if let Some(abs_position) = query.abs_position {
            fb = fb.add_single(abs_position.into());
        }
        if let Some(motor_temperature) = query.motor_temperature {
            fb = fb.add_single(motor_temperature.into());
        }
        if let Some(trajectory_complete) = query.trajectory_complete {
            fb = fb.add_single(trajectory_complete.into());
        }

        if let Some(home_state) = query.home_state {
            fb = fb.add_single(home_state.into());
        }
        if let Some(voltage) = query.voltage {
            fb = fb.add_single(voltage.into());
        }
        if let Some(temperature) = query.temperature {
            fb = fb.add_single(temperature.into());
        }
        if let Some(fault) = query.fault {
            fb = fb.add_single(fault.into());
        }
        if let Some(aux1_gpio) = query.aux1_gpio {
            fb = fb.add_single(aux1_gpio.into());
        }
        if let Some(aux2_gpio) = query.aux2_gpio {
            fb = fb.add_single(aux2_gpio.into());
        }
        for register in query.extra.unwrap_or_default() {
            fb = fb.add([register]);
        }
        fb
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let mut c = crate::Controller::default();
        let _ = c.query(1, QueryType::Default);
        let _ = c.query(1, QueryType::None);

        let custom = Frame::builder().add([registers::Mode::write(registers::Modes::Position).into()]);
        let _ = c.query(1, QueryType::Custom(custom));

        let custom = Frame::builder().add([registers::Mode::write(registers::Modes::Position).into()]);
        let _ = c.query(1, custom.into());
    }
}