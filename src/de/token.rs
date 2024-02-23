use std::{io, iter::Peekable};

use crate::error::{NeverFail, SyntaxError};

use super::position::{PosRange, Position};

pub struct Tokenizer<R>
where
    R: io::Read,
{
    iter: Peekable<RowColIterator<io::Bytes<R>>>,
}
impl<R> Tokenizer<R>
where
    R: io::Read,
{
    pub fn new(reader: R) -> Self {
        Tokenizer { iter: RowColIterator::new(reader.bytes()).peekable() }
    }
}
impl<R> Tokenizer<R>
where
    R: io::Read,
{
    pub fn eat(&mut self) -> crate::Result<Option<(Position, u8)>> {
        match self.iter.next() {
            Some((pos, Ok(c))) => Ok(Some((pos, c))),
            Some((_, Err(e))) => Err(crate::Error::new(e.to_string())), // TODO handling io error
            None => Ok(None),
        }
    }

    pub fn find(&mut self) -> crate::Result<Option<(Position, u8)>> {
        match self.iter.peek() {
            Some(&(pos, Ok(c))) => Ok(Some((pos, c))),
            Some((_, Err(e))) => Err(crate::Error::new(e.to_string())), // TODO handling io error
            None => Ok(None),
        }
    }

    pub fn eat_whitespace(&mut self) -> crate::Result<Option<(Position, u8)>> {
        while let Some((pos, c)) = self.eat()? {
            if !c.is_ascii_whitespace() {
                return Ok(Some((pos, c)));
            }
        }
        Ok(None)
    }

    pub fn skip_whitespace(&mut self) -> crate::Result<Option<(Position, u8)>> {
        while let Some((pos, c)) = self.find()? {
            if c.is_ascii_whitespace() {
                self.eat()?;
            } else {
                return Ok(Some((pos, c)));
            }
        }
        Ok(None)
    }

    pub fn fold_token<F: Fn(&[u8], u8) -> bool>(&mut self, f: F) -> crate::Result<(PosRange, Vec<u8>)> {
        let (start, _) = self.skip_whitespace()?.ok_or(SyntaxError::EofWhileParsingIdent)?;
        let (mut end, mut buff) = (start, Vec::new());
        while let Some((_pos, c)) = self.find()? {
            if f(&buff, c) {
                (end, _) = self.eat()?.ok_or(NeverFail::EatAfterFind)?;
                buff.push(c)
            } else {
                break;
            }
        }
        Ok(((start, end), buff))
    }

    pub fn parse_ident<T>(&mut self, ident: &[u8], value: T) -> crate::Result<T> {
        let max = 10; // to prevent from parsing tokens that are too long. the longest json ident is `false` of 5.
        let (pos, parsed) =
            self.fold_token(|b, c| b.len() < max && (c.is_ascii_alphanumeric() || ident.contains(&c)))?;
        if &parsed == ident {
            Ok(value)
        } else {
            Err(SyntaxError::UnexpectedIdent { pos, expected: ident.into(), found: parsed })?
        }
    }

    pub fn parse_string(&mut self) -> crate::Result<Vec<u8>> {
        let mut buff = Vec::new();
        match self.eat_whitespace()?.ok_or(SyntaxError::EofWhileStartParsingString)? {
            (_, b'"') => {
                self.parse_string_content(&mut buff)?;
                match self.eat()?.ok_or(SyntaxError::EofWhileEndParsingString)? {
                    (_, b'"') => Ok(buff),
                    (pos, found) => Err(SyntaxError::UnexpectedTokenWhileEndParsingString { pos, found })?,
                }
            }
            (pos, found) => Err(SyntaxError::UnexpectedTokenWhileStartParsingString { pos, found })?,
        }
    }

    pub fn parse_string_content(&mut self, buff: &mut Vec<u8>) -> crate::Result<()> {
        while let Some((pos, found)) = self.find()? {
            match found {
                b'\\' => self.parse_escape_sequence(buff)?,
                b'"' => return Ok(()),
                c if c.is_ascii_control() => Err(SyntaxError::ControlCharacterWhileParsingString { pos, c })?,
                _ => buff.push(self.eat()?.ok_or(NeverFail::EatAfterFind)?.1),
            }
        }
        Err(SyntaxError::EofWhileEndParsingString)? // TODO contain parsed string?
    }

    pub fn parse_escape_sequence(&mut self, buff: &mut Vec<u8>) -> crate::Result<()> {
        match self.eat()?.ok_or(SyntaxError::EofWhileParsingEscapeSequence)? {
            (_, b'\\') => match self.eat()?.ok_or(SyntaxError::EofWhileParsingEscapeSequence)? {
                (_, b'"') => Ok(buff.push(b'"')),
                (_, b'\\') => Ok(buff.push(b'\\')),
                (_, b'/') => Ok(buff.push(b'/')),
                (_, b'b') => Ok(buff.push(b'\x08')),
                (_, b'f') => Ok(buff.push(b'\x0C')),
                (_, b'n') => Ok(buff.push(b'\n')),
                (_, b'r') => Ok(buff.push(b'\r')),
                (_, b't') => Ok(buff.push(b'\t')),
                (_, b'u') => Ok(self.parse_unicode(buff)?),
                (pos, found) => Err(SyntaxError::InvalidEscapeSequence { pos, found })?,
            },
            (pos, found) => Err(SyntaxError::UnexpectedTokenWhileStartParsingEscapeSequence { pos, found })?,
        }
    }

    pub fn parse_unicode(&mut self, buff: &mut Vec<u8>) -> crate::Result<()> {
        let mut hex: u32 = 0;
        for i in 0..4 {
            match self.eat()?.ok_or(SyntaxError::EofWhileParsingEscapeSequence)? {
                (_, c @ b'0'..=b'9') => hex += ((c - b'0' + 0) as u32) << (3 - i) * 4,
                (_, c @ b'a'..=b'f') => hex += ((c - b'a' + 10) as u32) << (3 - i) * 4,
                (_, c @ b'A'..=b'F') => hex += ((c - b'A' + 10) as u32) << (3 - i) * 4,
                (pos, found) => return Err(SyntaxError::InvalidUnicodeEscape { pos, found })?,
            }
        }
        let ch = unsafe { char::from_u32_unchecked(hex) }; // TODO maybe safe
        Ok(buff.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()))
    }
}

pub struct RowColIterator<I> {
    iter: I,
    row: usize,
    col: usize,
}
impl<I> RowColIterator<I> {
    pub fn new(iter: I) -> Self {
        RowColIterator { iter, row: 0, col: 0 }
    }

    pub fn pos(&self) -> Position {
        (self.row, self.col)
    }
}
impl<I> Iterator for RowColIterator<I>
where
    I: Iterator<Item = io::Result<u8>>,
{
    type Item = (Position, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|c| {
            let pos = self.pos();
            match c {
                Ok(b'\n') => {
                    self.row += 1;
                    self.col = 0;
                }
                Ok(_) => self.col += 1,
                Err(_) => {}
            }
            (pos, c)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read};

    use super::*;

    #[test]
    fn behavior_row_col_iterator() {
        // [
        //   "foo",
        //   "bar",
        //   "baz"
        // ]
        //
        let raw = vec!["[", r#"  "foo","#, r#"  "bar","#, r#"  "baz""#, "]", ""].join("\n");
        let reader = BufReader::new(raw.as_bytes());
        let mut iter = RowColIterator::new(reader.bytes());
        assert!(matches!(iter.next(), Some(((0, 0), Ok(b'[')))));
        assert!(matches!(iter.next(), Some(((0, 1), Ok(b'\n')))));

        assert!(matches!(iter.next(), Some(((1, 0), Ok(b' ')))));
        assert!(matches!(iter.next(), Some(((1, 1), Ok(b' ')))));
        assert!(matches!(iter.next(), Some(((1, 2), Ok(b'"')))));
        assert!(matches!(iter.next(), Some(((1, 3), Ok(b'f')))));
        assert!(matches!(iter.next(), Some(((1, 4), Ok(b'o')))));
        assert!(matches!(iter.next(), Some(((1, 5), Ok(b'o')))));
        assert!(matches!(iter.next(), Some(((1, 6), Ok(b'"')))));
        assert!(matches!(iter.next(), Some(((1, 7), Ok(b',')))));
        assert!(matches!(iter.next(), Some(((1, 8), Ok(b'\n')))));

        assert!(matches!(iter.next(), Some(((2, 0), Ok(b' ')))));
        assert!(matches!(iter.next(), Some(((2, 1), Ok(b' ')))));
        assert!(matches!(iter.next(), Some(((2, 2), Ok(b'"')))));
        assert!(matches!(iter.next(), Some(((2, 3), Ok(b'b')))));
        assert!(matches!(iter.next(), Some(((2, 4), Ok(b'a')))));
        assert!(matches!(iter.next(), Some(((2, 5), Ok(b'r')))));
        assert!(matches!(iter.next(), Some(((2, 6), Ok(b'"')))));
        assert!(matches!(iter.next(), Some(((2, 7), Ok(b',')))));
        assert!(matches!(iter.next(), Some(((2, 8), Ok(b'\n')))));

        assert!(matches!(iter.next(), Some(((3, 0), Ok(b' ')))));
        assert!(matches!(iter.next(), Some(((3, 1), Ok(b' ')))));
        assert!(matches!(iter.next(), Some(((3, 2), Ok(b'"')))));
        assert!(matches!(iter.next(), Some(((3, 3), Ok(b'b')))));
        assert!(matches!(iter.next(), Some(((3, 4), Ok(b'a')))));
        assert!(matches!(iter.next(), Some(((3, 5), Ok(b'z')))));
        assert!(matches!(iter.next(), Some(((3, 6), Ok(b'"')))));
        assert!(matches!(iter.next(), Some(((3, 7), Ok(b'\n')))));

        assert!(matches!(iter.next(), Some(((4, 0), Ok(b']')))));
        assert!(matches!(iter.next(), Some(((4, 1), Ok(b'\n')))));

        assert!(matches!(iter.next(), None));
        assert!(matches!(iter.next(), None));
        assert!(matches!(iter.next(), None));
    }

    #[test]
    fn behavior_tokenizer() {
        let raw = r#"
            [
                "jsonc",
                123,
                true,
                false,
                null,
            ]
        "#;
        let reader = BufReader::new(raw.as_bytes());
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.find().unwrap(), Some(((0, 0), b'\n')));
        assert_eq!(tokenizer.find().unwrap(), Some(((0, 0), b'\n')));
        assert_eq!(tokenizer.eat().unwrap(), Some(((0, 0), b'\n')));
        assert_eq!(tokenizer.find().unwrap(), Some(((1, 0), b' ')));
        assert_eq!(tokenizer.find().unwrap(), Some(((1, 0), b' ')));
        assert_eq!(tokenizer.eat().unwrap(), Some(((1, 0), b' ')));

        assert_eq!(tokenizer.eat_whitespace().unwrap(), Some(((1, 12), b'[')));
        assert_eq!(tokenizer.find().unwrap(), Some(((1, 13), b'\n')));
        assert_eq!(tokenizer.skip_whitespace().unwrap(), Some(((2, 16), b'"')));
        assert_eq!(tokenizer.find().unwrap(), Some(((2, 16), b'"')));

        assert_eq!(tokenizer.parse_string().unwrap(), b"jsonc");
        assert!(matches!(tokenizer.eat(), Ok(Some((_, b',')))));

        assert!(matches!(tokenizer.skip_whitespace(), Ok(Some((_, b'1')))));
        assert!(matches!(tokenizer.parse_ident(b"123", 123), Ok(123)));
        assert!(matches!(tokenizer.eat(), Ok(Some((_, b',')))));

        assert!(matches!(tokenizer.skip_whitespace(), Ok(Some((_, b't')))));
        assert!(matches!(tokenizer.parse_ident(b"true", true), Ok(true)));
        assert!(matches!(tokenizer.eat(), Ok(Some((_, b',')))));

        assert!(matches!(tokenizer.skip_whitespace(), Ok(Some((_, b'f')))));
        assert!(matches!(tokenizer.parse_ident(b"false", false), Ok(false)));
        assert!(matches!(tokenizer.eat(), Ok(Some((_, b',')))));

        assert!(matches!(tokenizer.skip_whitespace(), Ok(Some((_, b'n')))));
        assert!(matches!(tokenizer.parse_ident(b"null", ()), Ok(())));
        assert!(matches!(tokenizer.eat(), Ok(Some((_, b',')))));

        assert_eq!(tokenizer.eat_whitespace().unwrap(), Some(((7, 12), b']')));
        assert_eq!(tokenizer.find().unwrap(), Some(((7, 13), b'\n')));
        assert_eq!(tokenizer.eat_whitespace().unwrap(), None);
    }

    #[test]
    fn test_parse_string() {
        let parse = |s: &str| Tokenizer::new(s.as_bytes()).parse_string();

        // ok
        assert_eq!(parse(r#""""#).unwrap(), b"");
        assert_eq!(parse(r#""rust""#).unwrap(), b"rust");
        assert_eq!(parse(r#""\"quote\"""#).unwrap(), b"\"quote\"");
        assert_eq!(parse(r#""back\\slash""#).unwrap(), b"back\\slash");
        assert_eq!(parse(r#""escaped\/slash""#).unwrap(), b"escaped/slash");
        assert_eq!(parse(r#""unescaped/slash""#).unwrap(), b"unescaped/slash");
        assert_eq!(parse(r#""backspace\b formfeed\f""#).unwrap(), b"backspace\x08 formfeed\x0C");
        assert_eq!(parse(r#""line\nfeed""#).unwrap(), b"line\nfeed");
        assert_eq!(parse(r#""white\tspace""#).unwrap(), b"white\tspace");
        assert_eq!(String::from_utf8(parse(r#""line\u000Afeed""#).unwrap()).unwrap(), "line\u{000A}feed");
        assert_eq!(parse(r#""line\u000Afeed""#).unwrap(), "line\nfeed".bytes().collect::<Vec<_>>());
        assert_eq!(parse(r#""epsilon \u03b5""#).unwrap(), "epsilon ε".bytes().collect::<Vec<_>>());
        assert_eq!(parse(r#""💯""#).unwrap(), "💯".bytes().collect::<Vec<_>>());

        // err
        assert!(matches!(
            parse(r#""ending..."#).unwrap_err().into_inner().downcast_ref().unwrap(),
            SyntaxError::EofWhileEndParsingString,
        ));
        assert!(matches!(
            parse(
                r#""line
                    feed""#
            )
            .unwrap_err()
            .into_inner()
            .downcast_ref()
            .unwrap(),
            SyntaxError::ControlCharacterWhileParsingString { c: b'\n', .. }
        ));
        assert!(matches!(
            parse(r#""escape EoF \"#).unwrap_err().into_inner().downcast_ref().unwrap(),
            SyntaxError::EofWhileParsingEscapeSequence,
        ));
        assert!(matches!(
            parse(r#""invalid escape sequence \a""#).unwrap_err().into_inner().downcast_ref().unwrap(),
            SyntaxError::InvalidEscapeSequence { found: b'a', .. }
        ));
        assert!(matches!(
            parse(r#""invalid unicode \uXXXX""#).unwrap_err().into_inner().downcast_ref().unwrap(),
            SyntaxError::InvalidUnicodeEscape { found: b'X', .. }
        ))
    }
}
