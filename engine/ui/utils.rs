use core::fmt::{self, Write};

use crate::prelude::*;

pub use shared::utils::*;

pub struct EscapeCommand<'a> {
    src: &'a str,
    scripting: bool,
}

impl<'a> EscapeCommand<'a> {
    pub fn new(src: &'a str, scripting: bool) -> Self {
        Self { src, scripting }
    }
}

impl fmt::Display for EscapeCommand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.src.chars() {
            match c {
                '"' => f.write_char('\\')?,
                '$' if self.scripting => f.write_char('$')?,
                _ => {}
            }
            f.write_char(c)?;
        }
        Ok(())
    }
}

pub fn escape_command(src: &str) -> EscapeCommand<'_> {
    EscapeCommand::new(src, engine().get_cvar(c"cmd_scripting"))
}

#[cfg(test)]
mod tests {
    use csz::CStrArray;

    use super::*;

    #[test]
    fn test_escape_command() {
        let cmd = EscapeCommand::new("abc \"123\" $x", true);
        let mut buf = CStrArray::<512>::new();
        write!(buf.cursor(), "{cmd}").unwrap();
        assert_eq!(cr#"abc \"123\" $$x"#, buf.as_c_str());
    }
}
