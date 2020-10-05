// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::writer::Writer,
    conquer_once::spin::Lazy,
    core::fmt::Write,
    log::{Level, LevelFilter, Metadata, Record, SetLoggerError},
    rgb::RGB8,
    spinning_top::Spinlock,
    vek::Vec2,
};

struct Logger;

static LOGGER: Logger = Logger;

static LOG_WRITER: Lazy<Spinlock<Writer>> =
    Lazy::new(|| Spinlock::new(Writer::new(Vec2::new(0, 100), RGB8::new(0xff, 0xff, 0xff))));

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

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
