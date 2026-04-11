use std::iter::{Iterator, Peekable};

pub struct DoublePeekable<I: Iterator> {
    peekable: Peekable<I>,
    peeked: Option<Option<I::Item>>,
}

impl<I: Iterator> DoublePeekable<I> {
    pub const fn new(peekable: Peekable<I>) -> DoublePeekable<I> {
        DoublePeekable {
            peekable,
            peeked: None,
        }
    }
}

impl<I: Iterator> Iterator for DoublePeekable<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        match self.peeked.take() {
            Some(v) => v,
            None => self.peekable.next(),
        }
    }

    #[inline]
    fn count(mut self) -> usize {
        match self.peeked.take() {
            Some(None) => 0,
            Some(Some(_)) => 1 + self.peekable.count(),
            None => self.peekable.count(),
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I::Item> {
        match self.peeked.take() {
            Some(None) => None,
            Some(v @ Some(_)) if n == 0 => v,
            Some(Some(_)) => self.peekable.nth(n - 1),
            None => self.peekable.nth(n),
        }
    }

    #[inline]
    fn last(mut self) -> Option<I::Item> {
        let peek_opt = match self.peeked.take() {
            Some(None) => return None,
            Some(v) => v,
            None => None,
        };
        self.peekable.last().or(peek_opt)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let peek_len = match self.peeked {
            Some(None) => return (0, Some(0)),
            Some(Some(_)) => 1,
            None => 0,
        };
        let (lo, hi) = self.peekable.size_hint();
        let lo = lo.saturating_add(peek_len);
        let hi = match hi {
            Some(x) => x.checked_add(peek_len),
            None => None,
        };
        (lo, hi)
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, mut fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let acc = match self.peeked {
            Some(None) => return init,
            Some(Some(v)) => fold(init, v),
            None => init,
        };
        self.peekable.fold(acc, fold)
    }
}

impl<'a, I: Iterator> DoublePeekable<I> {
    pub fn peek(&mut self) -> Option<&I::Item> {
        let iter = &mut self.peekable;
        self.peeked.get_or_insert_with(|| iter.next()).as_ref()
    }

    fn peek_next(&mut self) -> Option<&I::Item> {
        self.peekable.peek()
    }

    pub fn next_if(&mut self, func: impl FnOnce(&I::Item) -> bool) -> Option<I::Item> {
        match self.next() {
            Some(matched) if func(&matched) => Some(matched),
            other => {
                assert!(self.peeked.is_none());
                self.peeked = Some(other);
                None
            }
        }
    }

    pub fn next_if_eq<T>(&mut self, expected: &T) -> Option<I::Item>
    where
        T: ?Sized,
        I::Item: PartialEq<T>,
    {
        self.next_if(|next| next == expected)
    }

    pub fn peek_while<'b, F: FnMut(&I::Item) -> bool>(&'b mut self, f: F) -> PeekWhile<'a, I, F>
    where
        'b: 'a,
    {
        PeekWhile { iter: self, f }
    }
}

struct PeekWhile<'a, I: Iterator, F: FnMut(&I::Item) -> bool> {
    iter: &'a mut DoublePeekable<I>,
    f: F,
}

impl<'a, I: Iterator, F: FnMut(&I::Item) -> bool> Iterator for PeekWhile<'a, I, F> {
    type Item = I::Item;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let &mut PeekWhile {
            ref mut iter,
            ref mut f,
        } = self;
        if iter.peek().map(f).unwrap_or(false) {
            iter.next()
        } else {
            None
        }
    }
}

fn peek_while<'a, I, F>(iter: &'a mut DoublePeekable<I>, f: F) -> PeekWhile<'a, I, F>
where
    I: Iterator + 'a,
    F: for<'b> FnMut(&'b <I as Iterator>::Item) -> bool,
{
    PeekWhile { iter, f }
}
