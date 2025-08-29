use core::{
    fmt::{self, Write},
    marker::PhantomData,
};

use alloc::string::String;
use csz::{CStrArray, CStrThin};
use log::{Level, LevelFilter, Record};

const STACK_SIZE: usize = 8192;

pub trait EngineConsole: Send + Sync {
    fn get_cvar_float(name: &CStrThin) -> f32;

    fn console_print(s: &CStrThin);
}

struct ConsoleLogger<T>(PhantomData<T>);

impl<T: EngineConsole> ConsoleLogger<T> {
    fn log_console_stack(&self, level: &str, args: &fmt::Arguments) -> fmt::Result {
        let mut buffer = CStrArray::<STACK_SIZE>::new();
        let mut cur = buffer.cursor();
        fmt::Write::write_str(&mut cur, level)?;
        cur.write_fmt(*args)?;
        cur.write_char('\n')?;
        cur.finish();
        T::console_print(buffer.as_thin());
        Ok(())
    }

    fn log_console_heap(&self, level: &str, args: &fmt::Arguments) -> fmt::Result {
        let mut buffer = String::with_capacity(STACK_SIZE * 2);
        buffer.write_str(level)?;
        buffer.write_fmt(*args)?;
        buffer.write_str("\n\0")?;
        let msg = unsafe { CStrThin::from_bytes_until_nul_unchecked(buffer.as_bytes()) };
        T::console_print(msg.as_c_str().into());
        Ok(())
    }

    fn log_console(&self, level: &str, args: &fmt::Arguments) -> fmt::Result {
        self.log_console_stack(level, args)
            .or_else(|_| self.log_console_heap(level, args))
    }
}

impl<T: EngineConsole> log::Log for ConsoleLogger<T> {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let level = match record.level() {
            Level::Trace => "^6Trace:^7 ",
            Level::Debug => "^4Debug:^7 ",
            Level::Info => "",
            Level::Warn => "^3Warning:^7 ",
            Level::Error => "^1Error:^7 ",
        };
        if self.log_console(level, record.args()).is_err() {
            T::console_print(c"^1Error:^7 Failed to format log message".into());
        }
    }

    fn flush(&self) {}
}

pub fn init_console_logger<T: 'static + EngineConsole>() {
    let level = match T::get_cvar_float(c"developer".into()) as i32 {
        2 => LevelFilter::Trace,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };
    if log::set_logger(&ConsoleLogger(PhantomData::<T>)).is_err() {
        T::console_print(c"^1Error:^7 Failed initialize console logger".into());
    }
    log::set_max_level(level);
}
