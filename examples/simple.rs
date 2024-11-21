//! Based on the Python example [Simple] from the moteus library
//!
//!
//! This example commands a single servo at ID #1 using the default
//! transport to hold the current position indefinitely, and prints the
//! state of the servo to the console.
use fdcanusb::FdCanUSB;
use moteus::frame::QueryType;
use moteus::{registers, Controller};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env().init();
    // By default, the Controller connects to id 1, and picks an arbitrary
    // CAN-FD transport, prefering an attached fdcanusb if available.
    let mut c = crate::Controller::new(
        FdCanUSB::open("/dev/fdcanusb", fdcanusb::serial2::KeepSettings)?,
        false,
    );
    // In case the controller had faulted previously, at the start of
    // this script we send the stop command to clear it.
    c.send_no_response(1, moteus::frame::Stop)?;

    loop {
        // moteus::frame::Position has a number of helper methods
        // which return predefined position commands.  Here we use
        // the `hold` method to create a command which will hold the
        // current position of the controller.
        //

        // The first argument to `send_with_query` is the id of the
        // controller to send the command to.  The second argument is
        // the command to send.  The third argument is the query type
        // to use.  The query type can be one of `QueryType::Default`,
        // `QueryType::DefaultAnd`, or `QueryType::Custom`. This sets
        // which registers are returned in the response.
        //
        // The `send_with_query` method sends a command to the controller
        // and waits for a response. A `ResponseFrame` is returned which
        // contains the values of the registers requested in the query type.
        let state = c.send_with_query(1, moteus::frame::Position::hold(), QueryType::Default)?;
        // Print out everything.
        log::debug!("{:?}", state);

        // To retrieve the values of the registers from the `ResponseFrame`,
        // use the `get` method with the register type as the type parameter.
        // The `get` method returns a `Option` as the register may not be present
        // in the `ResponseFrame`.
        let pos = state.get::<registers::Position>();
        log::info!("Position: {:?}\n", pos);

        // Wait 20ms between iterations.  By default, when commanded
        // over CAN, there is a watchdog which requires commands to be
        // sent at least every 100ms or the controller will enter a
        // latched fault state.
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
