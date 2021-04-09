// SPDX-License-Identifier: GPL-3.0-or-later

use super::writer::Writer;
use core::{fmt, fmt::Write};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use rgb::RGB8;
use spinning_top::Spinlock;

static LOGGER: Logger = Logger;

static LOG_WRITER: Spinlock<Writer> = Spinlock::new(Writer::new(RGB8::new(0xff, 0xff, 0xff)));

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::log::_print(core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println{
    () => {
        print!("\n");
    };
    ($($arg:tt)*)=>{
        print!("{}\n",core::format_args!($($arg)*));
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    write!(*LOG_WRITER.lock(), "{}", args).unwrap();
}

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
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
