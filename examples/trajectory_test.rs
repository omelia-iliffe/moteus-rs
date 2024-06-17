//! Based on the Python example [Trajectory Test] from the moteus library
//!
//!
//! Demonstrates how to specify alternate registers to query, and how
//! to control the velocity and acceleration limits on a per-command basis
//! to create a continuous trajectory.
//!
use moteus::frame::{Query, QueryType};
use moteus::registers::*;
use moteus::{registers, Controller};

fn main() -> Result<(), moteus::Error> {
    env_logger::Builder::from_default_env().init();

    let qr = Query::new_with_extra([
        ControlPosition::read().into(),
        ControlVelocity::read().into(),
        ControlTorque::read().into(),
        ControlPositionError::read().into(),
        ControlVelocityError::read().into(),
        ControlTorqueError::read().into(),
    ]);
    // By default, Controller connects to id 1, and picks an arbitrary
    // CAN-FD transport, prefering an attached fdcanusb if available
    let mut transport =
        fdcanusb::FdCanUSB::open("/dev/fdcanusb", fdcanusb::serial2::KeepSettings).unwrap();
    transport.flush().unwrap();
    let mut c = Controller::with_query(transport, false, qr);
    // In case the controller had faulted previously, at the start of
    // this script we send the stop command in order to clear it
    c.send_no_response(1, moteus::frame::Stop).unwrap();

    let elapsed = std::time::Instant::now();

    loop {
        // `set_position` accepts an optional keyword argument for each
        // possible position mode register as described in the moteus
        // reference manual.  If a given register is omitted, then that
        // register is omitted from the command itself, with semantics
        // as described in the reference manual.
        //
        // The return type of 'set_position' is a moteus.Result type.
        // It has a __repr__ method, and has a 'values' field which can
        // be used to examine individual result registers.
        let position = if elapsed.elapsed().as_secs() % 2 == 0 {
            Some(CommandPosition::write(-0.5))
        } else {
            Some(CommandPosition::write(0.5))
        };
        let command = moteus::frame::Position {
            position,
            velocity: Some(CommandVelocity::write(0.0)),
            velocity_limit: Some(VelocityLimit::write(8.0)),
            acceleration_limit: Some(AccelerationLimit::write(3.0)),
            ..Default::default()
        };

        // Print out everything.
        let state = c.send_with_query(1, command, QueryType::Default)?;
        // Print out just the position register.
        log::debug!("{:?}", state);
        log::info!("Position: {:?}\n", state.get::<registers::Position>());

        // Wait 20ms between iterations.  By default, when commanded
        // over CAN, there is a watchdog which requires commands to be
        // sent at least every 100ms or the controller will enter a
        // latched fault state
        std::thread::sleep(std::time::Duration::from_millis(20));
        // }
    }
}
