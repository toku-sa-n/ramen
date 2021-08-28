// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::string::ToString;
use core::{convert::TryInto, fmt};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::_print(core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println{
    () => {
        print!("\n");
    };
    ($($arg:tt)*)=>{
        print!("{}\n",core::format_args!($($arg)*))
    }
}

pub(crate) fn init() {
    let r = log::set_logger(&LOGGER).map(|_| log::set_max_level(log::LevelFilter::Info));
    r.expect("Failed to initialize logger.");
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments<'_>) {
    let s = args.to_string();

    unsafe { syscalls::write(1, s.as_ptr().cast(), s.len().try_into().unwrap()) };
}

static LOGGER: Logger = Logger;
struct Logger;
impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        println!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
