//! Based on the Python example [Simple] from the moteus library
//!
//!
//! This example commands a single servo at ID #1 using the default
//! transport to hold the current position indefinitely, and prints the
//! state of the servo to the console.
use moteus::frame::QueryType;
use moteus::{registers, Controller};

mod _logging;

fn main() {
    _logging::init(env!("CARGO_CRATE_NAME"), 3);
    // By default, Controller connects to id 1, and picks an arbitrary
    // CAN-FD transport, prefering an attached fdcanusb if available.
    let mut c = Controller::default();
    // In case the controller had faulted previously, at the start of
    // this script we send the stop command in order to clear it.
    c.send(1, moteus::frame::Stop, QueryType::None).unwrap();

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
        // state = await c.set_position(position=math.nan, query=True)
        let state = c
            .send(1, moteus::frame::Position::hold(), QueryType::Default)
            .unwrap()
            .expect("No response");
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
