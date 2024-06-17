//! Based on the Python example [Simple] from the moteus library
//!
//!
//! This example commands a single servo at ID #1 using the default
//! transport to hold the current position indefinitely, and prints the
//! state of the servo to the console.
use fdcanusb::FdCanUSB;
use moteus::frame::QueryType;
use moteus::{registers, Controller};

fn main() -> Result<(), moteus::Error> {
    env_logger::Builder::from_default_env().init();
    // By default, the Controller connects to id 1, and picks an arbitrary
    // CAN-FD transport, prefering an attached fdcanusb if available.
    let mut c = crate::Controller::new(
        FdCanUSB::open("/dev/fdcanusb", fdcanusb::serial2::KeepSettings)?,
        false,
    );
    // In case the controller had faulted previously, at the start of
    // this script we send the stop command to clear it.
    c.send_no_response(1, moteus::frame::Stop).unwrap();

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
        let state = c.send_with_query(1, moteus::frame::Position::hold(), QueryType::Default)?;
        // Print out everything.
        log::debug!("{:?}", state);
        // Print out just the position register.
        log::info!("Position: {:?}\n", state.get::<registers::Position>());

        // Wait 20ms between iterations.  By default, when commanded
        // over CAN, there is a watchdog which requires commands to be
        // sent at least every 100ms or the controller will enter a
        // latched fault state.
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
