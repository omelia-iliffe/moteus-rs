//! A rust implementation of the Moteus Protocol. Used to communicate with Moteus controllers ([moteus-r4](https://mjbots.com/products/moteus-r4-11), [moteus-n1](https://mjbots.com/products/moteus-n1)) over CAN-FD.

#![deny(
bad_style,
dead_code,
improper_ctypes,
non_shorthand_field_patterns,
no_mangle_generic_items,
overflowing_literals,
path_statements,
patterns_in_fns_without_body,
// private_in_public,
unconditional_recursion,
unused,
unused_allocation,
unused_comparisons,
unused_parens,
while_true
)]
#![deny(
// missing_debug_implementations,
missing_docs,
trivial_casts,
trivial_numeric_casts,
unused_extern_crates,
unused_import_braces,
unused_qualifications,
unused_results
)]
#![warn(clippy::unwrap_used)]

mod bus;
pub mod frame;
mod protocol;

pub use bus::{Controller, Error};
pub use fdcanusb::FdCanUSB;
pub use protocol::{registers, Frame, Resolution, FrameBuilder};
