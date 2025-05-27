use core::fmt::{self, Write};

use crate::engine;

pub use utils::*;

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

pub fn escape_command(src: &str) -> EscapeCommand {
    EscapeCommand::new(src, engine().cvar(c"cmd_scripting"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_command() {
        let s = EscapeCommand::new("abc \"123\" $x", true).to_string();
        assert_eq!(s, r#"abc \"123\" $$x"#);
    }
}
