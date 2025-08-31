use core::{fmt, mem};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenError<'a> {
    InvalidData,
    UnexpectedToken(&'a str),
    UnexpectedEnd,
}

impl fmt::Display for TokenError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidData => write!(f, "Invalid data"),
            Self::UnexpectedToken(token) => write!(f, "Unexpected token \"{token}\""),
            Self::UnexpectedEnd => write!(f, "Unexpected end"),
        }
    }
}

pub struct Tokens<'a> {
    data: &'a str,
    hash_comments: bool,
    handle_bracket: bool,
    handle_colon: bool,
}

impl<'a> Tokens<'a> {
    pub fn new(data: &'a str) -> Tokens<'a> {
        Tokens {
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

    fn is_single_char(&self, c: char) -> bool {
        match c {
            '{' | '}' | '\'' | ',' => true,
            '(' | ')' if self.handle_bracket => true,
            ':' if self.handle_colon => true,
            _ => false,
        }
    }

    fn is_whitespace(c: char) -> bool {
        c <= ' '
    }

    fn skip_line(&mut self) {
        self.data = self.data.trim_start_matches(|c| c != '\n');
    }

    fn skip_whitespace(&mut self) {
        while !self.data.is_empty() {
            self.data = self.data.trim_start_matches(Self::is_whitespace);
            if self.data.starts_with("//") {
                self.skip_line();
                continue;
            }
            if self.hash_comments && self.data.starts_with("#") {
                self.skip_line();
                continue;
            }
            break;
        }
    }

    pub fn parse(&mut self) -> Result<&'a str, TokenError<'a>> {
        self.skip_whitespace();

        match self.data.chars().next().ok_or(TokenError::UnexpectedEnd)? {
            '"' => {
                let mut skip = false;
                for (i, c) in self.data.char_indices().skip(1) {
                    if skip {
                        skip = false;
                    } else if c == '\\' {
                        skip = true;
                    } else if c == '\"' {
                        let s = &self.data[1..i];
                        self.data = &self.data[i + 1..];
                        return Ok(s);
                    }
                }
                Err(TokenError::InvalidData)
            }
            c if self.is_single_char(c) => {
                let s = &self.data[..1];
                self.data = &self.data[1..];
                Ok(s)
            }
            _ => match self
                .data
                .char_indices()
                .find(|&(_, c)| Self::is_whitespace(c) || self.is_single_char(c))
            {
                Some((i, _)) => {
                    let (head, tail) = self.data.split_at(i);
                    self.data = tail;
                    Ok(head)
                }
                None => Ok(mem::take(&mut self.data)),
            },
        }
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
