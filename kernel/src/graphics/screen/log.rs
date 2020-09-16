// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::writer::Writer,
    conquer_once::spin::Lazy,
    core::fmt::Write,
    log::{LevelFilter, Metadata, Record, SetLoggerError},
    rgb::RGB8,
    spinning_top::Spinlock,
    vek::Vec2,
};

struct Logger;

static LOGGER: Logger = Logger;

static LOG_WRITER: Lazy<Spinlock<Writer>> =
    Lazy::new(|| Spinlock::new(Writer::new(Vec2::new(0, 100), RGB8::new(0xff, 0xff, 0xff))));

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        writeln!(*LOG_WRITER.lock(), "{} - {}", record.level(), record.args()).unwrap();
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
