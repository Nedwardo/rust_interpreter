pub struct StringIter<'a> {
    pub string: &'a str,
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

pub struct StringTake<'b> {
    iter: StringIter<'b>,
    n: usize,
}

impl<'b> StringTake<'b> {
    pub fn new(iter: StringIter<'b>, n: usize) -> StringTake<'b> {
        StringTake { iter, n }
    }
    pub fn as_str(&mut self) -> Option<&'b str> {
        // SAFETY: `StringIter` is only made from a str, which guarantees the iter is valid UTF-8.
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
