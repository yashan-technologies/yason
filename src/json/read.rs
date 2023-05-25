pub struct SliceRead<'a> {
    slice: &'a [u8],
    index: usize,
    line: usize,
    column: usize,
}

impl<'a> SliceRead<'a> {
    #[inline]
    pub const fn new(slice: &'a [u8]) -> Self {
        Self {
            slice,
            index: 0,
            line: 1,
            column: 0,
        }
    }

    #[inline]
    pub fn next(&mut self) -> Option<u8> {
        if self.index < self.slice.len() {
            let ch = self.slice[self.index];
            self.advance();
            Some(ch)
        } else {
            None
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<u8> {
        if self.index < self.slice.len() {
            Some(self.slice[self.index])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_interval(&self, start: usize, end: usize) -> Option<&'a [u8]> {
        if end <= self.slice.len() && end >= start {
            Some(&self.slice[start..end])
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn advance(&mut self) {
        self.index += 1;
        self.column += 1;
    }

    #[inline(always)]
    pub fn step_forward(&mut self, step: usize) {
        self.index += step;
        self.column += step;
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
        }
    }

    #[inline]
    pub fn peek_position(&self) -> Position {
        let mut step = 0;
        if self.index < self.slice.len() {
            step = 1;
        }

        Position {
            line: self.line,
            column: self.column + step,
        }
    }

    #[inline]
    pub fn new_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }
}

pub struct Position {
    pub line: usize,
    pub column: usize,
}
