# Version 0.3.1 - 06-09-2024
- **Minor**: Fixed `disable_brs` field not doing anything
- **Minor**: Added feature `aux_index_raw` to read the custom aux_index_raw register, see my fork of the moteus firmware for more information.
# Version 0.3.0 - 04-09-2024
- **Major**: Added `Write`, `Read` and `Res` wrapper types to improve ergonomics.
- **Major**: Changed `write` method to correctly return a `Result` instead of panicking or creating invariants.
- **Major**: Renamed `FrameBuilder::add_register` to `FrameBuilder::add` and added `try_add_many` method to add multiple registers at once.
# Version 0.2.1 - 20-08-2024
- **Minor**: Updated fdcanusb to 0.6.0
# Version 0.2.0 - 09-08-2024
- **Major**: Added error.rs and error types to reduce usage of std::io::Error.
- **Major**: Bumped fdcanusb to 0.5.0
# Version 0.1.0 - 20-05-2024
- Initial release