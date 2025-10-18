use core::{
    fmt::{self, Write},
    marker::PhantomData,
    ptr,
};

use alloc::{boxed::Box, string::String, vec::Vec};
use csz::{CStrArray, CStrThin};
use log::{Level, LevelFilter, Record};

const STACK_SIZE: usize = 8192;

struct Filter {
    level: LevelFilter,
    directives: Vec<(Box<str>, LevelFilter)>,
}

impl Filter {
    const fn new() -> Self {
        Self {
            level: LevelFilter::Off,
            directives: Vec::new(),
        }
    }

    fn init(&mut self, level: LevelFilter) {
        self.level = level;
        self.directives.clear();
    }

    fn parse(&mut self, filter: &str) {
        for i in filter.split(',') {
            if let Ok(level) = i.parse::<LevelFilter>() {
                self.level = level;
                continue;
            }

            let mut it = i.split('=');
            let Some(target) = it.next() else { continue };
            let level = it
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(LevelFilter::Trace);
            self.directives.push((Box::from(target), level));
        }
    }

    fn enabled(&self, meta: &log::Metadata) -> bool {
        if !self.directives.is_empty() {
            for (target, level) in &self.directives {
                if meta.target().contains(target.as_ref()) {
                    return meta.level() <= *level;
                }
            }
            meta.level() <= self.level
        } else {
            true
        }
    }

    fn max_level(&self) -> LevelFilter {
        match self.directives.iter().max_by_key(|(_, l)| l) {
            Some((_, l)) => self.level.max(*l),
            None => self.level,
        }
    }
}

pub trait EngineConsoleLogger: Send + Sync {
    /// Print string to the console.
    ///
    /// # Safety
    ///
    /// Must be called only from the main engine thread.
    unsafe fn console_print(s: &CStrThin);
}

struct ConsoleLogger<T>(PhantomData<T>);

impl<T: EngineConsoleLogger> ConsoleLogger<T> {
    fn write<W: Write>(&self, record: &Record, f: &mut W) -> fmt::Result {
        f.write_char('[')?;
        match record.level() {
            Level::Trace => f.write_str("^6TRACE")?,
            Level::Debug => f.write_str("^4DEBUG")?,
            Level::Info => f.write_str("^2INFO")?,
            Level::Warn => f.write_str("^3WARNING")?,
            Level::Error => f.write_str("^1ERROR")?,
        };
        if record.level() != Level::Info {
            f.write_str(" ^7")?;
            f.write_str(record.target())?;
            f.write_str("] ")?;
        } else {
            f.write_str("^7]")?;
        }
        f.write_fmt(*record.args())?;
        f.write_char('\n')?;
        Ok(())
    }

    fn print(&self, s: &CStrThin) {
        // FIXME: can be called from other threads
        unsafe {
            T::console_print(s);
        }
    }

    fn log_console_stack(&self, record: &Record) -> fmt::Result {
        let mut buffer = CStrArray::<STACK_SIZE>::new();
        self.write(record, &mut buffer.cursor())?;
        self.print(buffer.as_thin());
        Ok(())
    }

    fn log_console_heap(&self, record: &Record) -> fmt::Result {
        let mut buffer = String::with_capacity(STACK_SIZE * 2);
        self.write(record, &mut buffer)?;
        buffer.push('\0');
        self.print(unsafe { CStrThin::from_ptr(buffer.as_bytes().as_ptr().cast()) });
        Ok(())
    }

    fn log_console(&self, record: &Record) -> fmt::Result {
        self.log_console_stack(record)
            .or_else(|_| self.log_console_heap(record))
    }
}

impl<T: EngineConsoleLogger> log::Log for ConsoleLogger<T> {
    fn enabled(&self, meta: &log::Metadata) -> bool {
        let filter = unsafe { &*ptr::addr_of_mut!(FILTER) };
        filter.enabled(meta)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if self.log_console(record).is_err() {
            self.print(c"[^1Error^7] Failed to format log message".into());
        }
    }

    fn flush(&self) {}
}

static mut FILTER: Filter = Filter::new();

/// Initialize logger.
///
/// # Safety
///
/// Must be called only from the main engine thread.
pub unsafe fn init_console_logger<T>(developer: f32, filter: Option<&CStrThin>)
where
    T: 'static + EngineConsoleLogger,
{
    let level = match developer as i32 {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let global_filter = unsafe { &mut *ptr::addr_of_mut!(FILTER) };
    global_filter.init(level);

    if let Some(filter) = filter {
        match filter.to_str() {
            Ok(filter) => global_filter.parse(filter),
            Err(_) => {
                let err = c"[^1Error^7] Failed initialize console logger filter".into();
                // SAFETY: called from the main game thread
                unsafe {
                    T::console_print(err);
                }
            }
        }
    }

    if log::set_logger(&ConsoleLogger(PhantomData::<T>)).is_err() {
        // SAFETY: called from the main game thread
        unsafe {
            T::console_print(c"[^1Error^7] Failed initialize console logger".into());
        }
    }
    log::set_max_level(global_filter.max_level());
}
