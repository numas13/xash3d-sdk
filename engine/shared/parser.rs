use core::{fmt, mem, str};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenError<'a> {
    InvalidData,
    UnexpectedTokenBytes(&'a [u8]),
    UnexpectedToken(&'a str),
    UnexpectedEnd,
}

impl fmt::Display for TokenError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidData => write!(f, "Invalid data"),
            Self::UnexpectedTokenBytes(token) => write!(f, "Unexpected token \"{token:?}\""),
            Self::UnexpectedToken(token) => write!(f, "Unexpected token \"{token}\""),
            Self::UnexpectedEnd => write!(f, "Unexpected end"),
        }
    }
}

pub struct TokensBytes<'a> {
    data: &'a [u8],
    hash_comments: bool,
    handle_bracket: bool,
    handle_colon: bool,
}

impl<'a> TokensBytes<'a> {
    pub fn new(data: &'a [u8]) -> TokensBytes<'a> {
        Self {
            data,
            hash_comments: true,
            handle_bracket: true,
            handle_colon: true,
        }
    }

    pub fn hash_comments(mut self, hash_comments: bool) -> Self {
        self.hash_comments = hash_comments;
        self
    }

    pub fn handle_bracket(mut self, bracket: bool) -> Self {
        self.handle_bracket = bracket;
        self
    }

    pub fn handle_colon(mut self, colon: bool) -> Self {
        self.handle_colon = colon;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn is_single_char(&self, c: u8) -> bool {
        match c {
            b'{' | b'}' | b'\'' | b',' => true,
            b'(' | b')' if self.handle_bracket => true,
            b':' if self.handle_colon => true,
            _ => false,
        }
    }

    fn trim_start_matches(&mut self, f: impl Fn(u8) -> bool) {
        let offset = self
            .data
            .iter()
            .position(|&i| !f(i))
            .unwrap_or(self.data.len());
        if offset != 0 {
            self.data = &self.data[offset..];
        }
    }

    fn skip_line(&mut self) {
        self.trim_start_matches(|c| c != b'\n');
    }

    fn is_whitespace(c: u8) -> bool {
        c <= b' '
    }

    fn skip_whitespace(&mut self) {
        while !self.data.is_empty() {
            self.trim_start_matches(Self::is_whitespace);
            if self.data.starts_with(b"//") {
                self.skip_line();
                continue;
            }
            if self.hash_comments && self.data.starts_with(b"#") {
                self.skip_line();
                continue;
            }
            break;
        }
    }

    pub fn parse(&mut self) -> Result<&'a [u8], TokenError<'a>> {
        self.skip_whitespace();

        match self.data.first().ok_or(TokenError::UnexpectedEnd)? {
            b'"' => {
                let mut skip = false;
                for (i, &c) in self.data.iter().enumerate().skip(1) {
                    if skip {
                        skip = false;
                    } else if c == b'\\' {
                        skip = true;
                    } else if c == b'\"' {
                        let s = &self.data[1..i];
                        self.data = &self.data[i + 1..];
                        return Ok(s);
                    }
                }
                Err(TokenError::InvalidData)
            }
            c if self.is_single_char(*c) => {
                let s = &self.data[..1];
                self.data = &self.data[1..];
                Ok(s)
            }
            _ => match self
                .data
                .iter()
                .position(|&c| Self::is_whitespace(c) || self.is_single_char(c))
            {
                Some(i) => {
                    let (head, tail) = self.data.split_at(i);
                    self.data = tail;
                    Ok(head)
                }
                None => Ok(mem::take(&mut self.data)),
            },
        }
    }

    pub fn expect(&mut self, token: &[u8]) -> Result<&'a [u8], TokenError<'a>> {
        let s = self.parse()?;
        if s == token {
            Ok(s)
        } else {
            Err(TokenError::UnexpectedTokenBytes(s))
        }
    }
}

impl<'a> Iterator for TokensBytes<'a> {
    type Item = Result<&'a [u8], TokenError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(s) => Some(Ok(s)),
            Err(TokenError::UnexpectedEnd) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct Tokens<'a> {
    bytes: TokensBytes<'a>,
}

impl<'a> Tokens<'a> {
    pub fn new(data: &'a str) -> Tokens<'a> {
        Self {
            bytes: TokensBytes::new(data.as_bytes()),
        }
    }

    pub fn hash_comments(self, hash_comments: bool) -> Self {
        Self {
            bytes: self.bytes.hash_comments(hash_comments),
        }
    }

    pub fn handle_bracket(self, bracket: bool) -> Self {
        Self {
            bytes: self.bytes.handle_bracket(bracket),
        }
    }

    pub fn handle_colon(self, colon: bool) -> Self {
        Self {
            bytes: self.bytes.handle_colon(colon),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn parse(&mut self) -> Result<&'a str, TokenError<'a>> {
        self.bytes.parse().map(|s| {
            // SAFETY: bytes are valid utf8 string
            unsafe { str::from_utf8_unchecked(s) }
        })
    }

    pub fn expect(&mut self, token: &str) -> Result<&'a str, TokenError<'a>> {
        let s = self.parse()?;
        if s == token {
            Ok(s)
        } else {
            Err(TokenError::UnexpectedToken(s))
        }
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Result<&'a str, TokenError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(s) => Some(Ok(s)),
            Err(TokenError::UnexpectedEnd) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub fn tokens_bytes(data: &[u8]) -> TokensBytes<'_> {
    TokensBytes::new(data)
}

pub fn tokens(data: &str) -> Tokens<'_> {
    Tokens::new(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace() {
        let src = " \t\n// foobar\n# 1234\n\n\"te\\\"st\"";
        assert_eq!(tokens(src).parse(), Ok("te\\\"st"));
    }

    #[test]
    fn single_char() {
        assert_eq!(tokens(" {\nfoobar").parse(), Ok("{"));
        assert_eq!(tokens(" }\nfoobar").parse(), Ok("}"));
        assert_eq!(tokens("\'\nfoobar").parse(), Ok("\'"));
        assert_eq!(tokens(" ,\nfoobar").parse(), Ok(","));
        assert_eq!(tokens(" (\nfoobar").parse(), Ok("("));
        assert_eq!(tokens(" )\nfoobar").parse(), Ok(")"));
        assert_eq!(tokens(" :\nfoobar").parse(), Ok(":"));
    }

    #[test]
    fn word() {
        assert_eq!(tokens("abc\nabc").parse(), Ok("abc"));
    }

    #[test]
    fn test1() {
        let data = r#"
            "lang" {
                "Language" "English"
                "Tokens" {
                    "Valve_Listen_MapName" "Map"
                    "Valve_Movement_Title" "MOVEMENT"
                }
            }
        "#;

        let expect = [
            "lang",
            "{",
            "Language",
            "English",
            "Tokens",
            "{",
            "Valve_Listen_MapName",
            "Map",
            "Valve_Movement_Title",
            "MOVEMENT",
            "}",
            "}",
        ];

        let mut iter = Tokens::new(data);
        for i in expect {
            println!("{i}");
            assert_eq!(iter.parse(), Ok(i));
        }
        assert_eq!(iter.parse(), Err(TokenError::UnexpectedEnd));
    }

    #[test]
    fn parse_colon() {
        let data = "abc:123 foobar";
        let mut tokens = Tokens::new(data).handle_colon(false);
        assert_eq!(tokens.parse(), Ok("abc:123"));
    }
}
