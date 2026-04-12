use crate::double_peekable::DoublePeekable;
use std::iter::Peekable;

pub struct StringIter<'a> {
    pub string: &'a str,
    pub peeked: Option<&'a char>,
}

impl<'a> Iterator for StringIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let mut chars = self.string.chars();
        let char = chars.next()?;
        self.string = chars.as_str();
        Some(char)
    }
}

impl<'a> Peekable for StringIter<'a> {}

pub struct StringTake<'b> {
    iter: DoublePeekable<StringIter<'b>>,
    n: usize,
}

impl<'b> StringTake<'b> {
    pub fn from_double_peekable(iter: DoublePeekable<StringIter<'b>>, n: usize) -> StringTake<'b> {
        StringTake { iter, n }
    }
    pub fn as_str(&mut self) -> Option<&'b str> {
        let start = self.iter.string;

        for _ in 0..self.n {
            self.iter.next();
        }

        if start.len() == 0 {
            None
        } else {
            let bytes_consumed = start.len() - self.iter.string.len();
            Some(&start[..bytes_consumed])
        }
    }
}
