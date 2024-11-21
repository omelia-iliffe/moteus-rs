//! Based on the Python example [Trajectory Test] from the moteus library
//!
//!
//! Demonstrates how to specify alternate registers to query, and how
//! to control the velocity and acceleration limits on a per-command basis
//! to create a continuous trajectory.
use moteus::frame::{Query, QueryType};
use moteus::registers::*;
use moteus::Controller;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env().init();

    let qr = Query::new_with_extra([
        ControlPosition::read().into(),
        ControlVelocity::read().into(),
        ControlTorque::read().into(),
        ControlPositionError::read().into(),
        ControlVelocityError::read().into(),
        ControlTorqueError::read().into(),
    ]);

    let mut transport =
        fdcanusb::FdCanUSB::open("/dev/fdcanusb", fdcanusb::serial2::KeepSettings).unwrap();
    transport.flush()?;
    let mut c = Controller::with_query(transport, false, qr);
    // In case the controller had faulted previously, at the start of
    // this script we send the stop command in order to clear it
    c.send_no_response(1, moteus::frame::Stop).unwrap();

    let elapsed = std::time::Instant::now();

    loop {
        // moteus::frame::Position can be constructed with the registers
        // you would like to write. Here we alternate between commanding
        // the motor to -0.5 and 0.5 radians every 2 seconds.
        let position = if elapsed.elapsed().as_secs() % 2 == 0 {
            CommandPosition::write(-0.5)
        } else {
            CommandPosition::write(0.5)
        }?;
        let command = moteus::frame::Position {
            position: Some(position),
            velocity: Some(CommandVelocity::write(0.0)?),
            velocity_limit: Some(VelocityLimit::write(8.0)?),
            acceleration_limit: Some(AccelerationLimit::write(3.0)?),
            ..Default::default()
        };

        // The first argument to `send_with_query` is the id of the
        // controller to send the command to.  The second argument is
        // the command to send.  The third argument is the query type
        // to use.  The query type can be one of `QueryType::Default`,
        // `QueryType::DefaultAnd`, or `QueryType::Custom`. This sets
        // which registers are returned in the response.
        //
        // The `send_with_query` method sends a command to the controller
        // and waits for a response. A `ResponseFrame` is returned which
        // contains the values of the registers requested in the query type
        let state = c.send_with_query(1, command, QueryType::Default)?;
        // Print out just the position register.
        log::debug!("{:?}", state);
        log::info!("Position: {:?}\n", state.get::<Position>());

        // Wait 20ms between iterations.  By default, when commanded
        // over CAN, there is a watchdog which requires commands to be
        // sent at least every 100ms or the controller will enter a
        // latched fault state
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
