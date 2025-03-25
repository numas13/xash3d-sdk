use core::fmt::Write;

use cell::SyncOnceCell;
use csz::{CStrArray, CStrThin};
use log::{Level, LevelFilter, Record};

struct Logger {
    write_fn: fn(&CStrThin),
}

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut buffer = CStrArray::<8192>::new();
            let level = match record.level() {
                Level::Trace => "^6Trace:^7 ",
                Level::Debug => "^4Debug:^7 ",
                Level::Info => "",
                Level::Warn => "^3Warning:^7 ",
                Level::Error => "^1Error:^7 ",
            };
            let mut cur = buffer.cursor();
            cur.write_str(level).unwrap();
            cur.write_fmt(*record.args()).unwrap();
            cur.write_str("\n").unwrap();
            cur.finish();
            (self.write_fn)(buffer.as_thin());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SyncOnceCell<Logger> = unsafe { SyncOnceCell::new() };

pub fn init(dev: i32, write_fn: fn(&CStrThin)) {
    let level = match dev {
        2 => LevelFilter::Trace,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };

    let logger = LOGGER.get_or_init(|| Logger { write_fn });
    log::set_logger(logger)
        .map(|_| log::set_max_level(level))
        .unwrap();
}
