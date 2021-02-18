// SPDX-License-Identifier: GPL-3.0-or-later

use super::writer::Writer;
use conquer_once::spin::Lazy;
use core::fmt::Write;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use rgb::RGB8;
use spinning_top::Spinlock;

struct Logger;

static LOGGER: Logger = Logger;

static LOG_WRITER: Lazy<Spinlock<Writer>> =
    Lazy::new(|| Spinlock::new(Writer::new(RGB8::new(0xff, 0xff, 0xff))));

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        write!(*LOG_WRITER.lock(),$($arg)*);
    };
}

#[macro_export]
macro_rules! println{
    ($($arg:tt)*)=>{
        writeln!(*LOG_WRITER.lock().$($arg)*);
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            writeln!(*LOG_WRITER.lock(), "{} - {}", record.level(), record.args()).unwrap();
        }
    }

    fn flush(&self) {}
}

/// # Errors
///
/// This function may return an error from `log::set_logger` function.
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
