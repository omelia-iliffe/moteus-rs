extern crate env_logger;

pub fn init(root_module: &str, verbosity: i8) {
    use std::io::Write;

    let log_level = match verbosity {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .format(|buffer, record: &log::Record| {
            use self::env_logger::fmt::Color;

            let mut prefix_style = buffer.style();
            let prefix;

            match record.level() {
                log::Level::Trace => {
                    prefix = "Trace: ";
                }
                log::Level::Debug => {
                    prefix = "";
                }
                log::Level::Info => {
                    prefix = "";
                }
                log::Level::Warn => {
                    prefix = "Warning: ";
                    prefix_style.set_color(Color::Yellow).set_bold(true);
                }
                log::Level::Error => {
                    prefix = "Error: ";
                    prefix_style.set_color(Color::Red).set_bold(true);
                }
            };

            writeln!(
                buffer,
                "{}:{} {} {} {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%H:%M:%S"),
                prefix_style.value(prefix),
                record.args(),
            )
        })
        .filter_level(log::LevelFilter::Warn)
        .filter_module(root_module, log_level)
        .filter_module("fdcanusb", log_level)
        .init();
}

#[allow(dead_code)]
fn main() {}
