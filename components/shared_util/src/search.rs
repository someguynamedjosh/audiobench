use scones::make_constructor;

#[make_constructor]
#[make_constructor(pub new_start)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    #[value(1 for new_start)]
    pub line: usize,
    #[value(1 for new_start)]
    pub column: usize,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)?;
        Ok(())
    }
}

impl Position {
    pub fn after_char(self, ch: char) -> Self {
        if ch == '\r' {
            // Ignore because \r is yucky.
            self
        } else if ch == '\n' {
            Self {
                line: self.line + 1,
                column: 1,
            }
        } else {
            Self {
                line: self.line,
                column: self.column + 1,
            }
        }
    }

    pub fn after_str(mut self, s: &str) -> Self {
        for c in s.chars() {
            self = self.after_char(c);
        }
        self
    }
}

/// Represents a segment of text text.
#[make_constructor]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clip {
    #[value(Position::new_start())]
    source_position: Position,
    text: String,
}

impl<T: Into<String>> From<T> for Clip {
    fn from(other: T) -> Self {
        Self::new(other.into())
    }
}

impl Clip {
    pub fn search(&self) -> ClipSearcher {
        ClipSearcher::from_clip(self)
    }

    pub fn get_source_position(&self) -> Position {
        self.source_position
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClipMarker<'a> {
    pos: Position,
    text: &'a str,
}

#[make_constructor]
#[derive(Clone, Copy, Debug)]
pub struct ClipSearcher<'a> {
    #[value(Position::new_start())]
    pos: Position,
    text: &'a str,
    start_pos: Position,
    original_text: &'a str,
}

impl<'a> ClipSearcher<'a> {
    pub fn from_clip(clip: &'a Clip) -> Self {
        Self {
            pos: clip.source_position,
            text: &clip.text,
            start_pos: clip.source_position,
            original_text: &clip.text,
        }
    }

    /// Returns how many characters away the searcher is from encountering the specified pattern.
    /// Returns None if that pattern does not occur in the remainder of the string.
    pub fn peek_pattern_start(&self, pattern: &str) -> Option<usize> {
        self.text.find(pattern)
    }

    pub fn peek_symbol_start(&self, pattern: &str) -> Option<usize> {
        let mut stext = self.text;
        let mut result = 0;
        while let Some(index) = stext.find(pattern) {
            result += index;
            let mut is_symbol = true;
            if index > 0 {
                let before = stext[index - 1..].chars().next().unwrap();
                if before.is_alphanumeric() {
                    is_symbol = false;
                }
            }
            if index + pattern.len() < stext.len() {
                let after = stext[index + pattern.len()..].chars().next().unwrap();
                if after.is_alphanumeric() {
                    is_symbol = false;
                }
            }
            if is_symbol {
                return Some(result);
            } else {
                result += 1;
                stext = &stext[index + 1..];
            }
        }
        None
    }

    pub fn skip_n(&mut self, n: usize) -> &mut Self {
        self.pos = self.pos.after_str(&self.text[..n]);
        self.text = &self.text[n..];
        self
    }

    pub fn goto_end(&mut self) -> &mut Self {
        self.skip_n(self.text.len())
    }

    pub fn goto_pattern_start(&mut self, pattern: &str) -> &mut Self {
        if let Some(index) = self.peek_pattern_start(pattern) {
            self.skip_n(index)
        } else {
            self.goto_end()
        }
    }

    pub fn goto_symbol_start(&mut self, pattern: &str) -> &mut Self {
        if let Some(index) = self.peek_symbol_start(pattern) {
            self.skip_n(index)
        } else {
            self.goto_end()
        }
    }

    pub fn goto_pattern_end(&mut self, pattern: &str) -> &mut Self {
        if let Some(index) = self.peek_pattern_start(pattern) {
            self.skip_n(index + pattern.len())
        } else {
            self.goto_end()
        }
    }

    pub fn skip_whitespace(&mut self) -> &mut Self {
        let mut to_skip = 0;
        for c in self.text.chars() {
            if c.is_whitespace() {
                to_skip += 1;
            } else {
                break;
            }
        }
        self.skip_n(to_skip)
    }

    fn delim_dists<'d>(&mut self, delimiters: &[(&str, &'d str)]) -> Vec<(usize, &'d str)> {
        delimiters
            .into_iter()
            .filter_map(|(start, end)| self.peek_symbol_start(start).map(|e| (e, *end)))
            .collect()
    }

    /// Skips over nested blocks with arbitrary delimiters until the terminator pattern is reached.
    /// If no terminator is found by the end, the result is the same as .goto_end(). For example,
    /// if you had a string "{ { term } } term" and you called this function as
    /// `skip_blocks(&[("{", "}")], "term")`, it would skip to the second occurence of "term".
    pub fn skip_blocks(&mut self, delimiters: &[(&str, &str)], terminator: &str) -> &mut Self {
        let mut stack = vec![terminator];
        while stack.len() > 0 {
            // How many more characters until we encounter something that can be popped off the
            // stack.
            let end_dist = if let Some(d) = self.peek_symbol_start(stack.last().unwrap()) {
                d
            } else {
                return self.goto_end();
            };
            // How many more characters until we encounter another starting delimiter.
            let delim_dists = self.delim_dists(delimiters);
            let mut min_dist = end_dist;
            // Set to some if an opening delimiter appears before the next opportunity to pop
            // something off the stack.
            let mut closest_delimiter = None;
            for (distance, end_delim) in delim_dists {
                if distance < min_dist {
                    min_dist = distance;
                    closest_delimiter = Some((distance, end_delim));
                }
            }
            if let Some((distance, end_delim)) = closest_delimiter {
                self.text = &self.text[distance..];
                stack.push(end_delim);
            } else {
                self.goto_symbol_start(stack.pop().unwrap());
            }
            if self.at_end() {
                return self;
            }
            // A bit hacky but it works to prevent consuming the same symbol several times.
            if stack.len() > 0 {
                self.text = &self.text[1..];
            }
        }
        self
    }

    pub fn start_clip(&self) -> ClipMarker<'a> {
        ClipMarker {
            pos: self.pos,
            text: self.text,
        }
    }

    pub fn end_clip(&self, start_marker: ClipMarker<'a>) -> Clip {
        Clip {
            source_position: start_marker.pos,
            text: String::from(&start_marker.text[..(start_marker.text.len() - self.text.len())]),
        }
    }

    pub fn tri_split(&self, start_marker: ClipMarker<'a>) -> (Clip, (Clip, Clip)) {
        let clipped = self.end_clip(start_marker);
        let original_len = self.original_text.len();
        let not_before_len = start_marker.text.len();
        let before = Clip {
            source_position: self.start_pos,
            text: String::from(&self.original_text[..(original_len - not_before_len)])
        };
        let after = Clip {
            source_position: self.pos,
            text: String::from(self.text)
        };
        (clipped, (before, after))
    }

    pub fn remaining(&self) -> &str {
        self.text
    }

    pub fn remaining_len(&self) -> usize {
        self.remaining().len()
    }

    pub fn at_end(&self) -> bool {
        self.remaining_len() == 0
    }

    pub fn get_current_pos(&self) -> Position {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn misc() {
        let clip = Clip::from("abcdef");
        let mut searcher = clip.search();
        searcher.goto_pattern_start("def");
        assert_eq!(searcher.remaining(), "def");
        let mut searcher = clip.search();
        searcher.goto_pattern_end("d");
        assert_eq!(searcher.remaining(), "ef");

        let mut searcher = clip.search();
        searcher.goto_pattern_start("g");
        assert_eq!(searcher.remaining(), "");
        let mut searcher = clip.search();
        searcher.goto_pattern_end("g");
        assert_eq!(searcher.remaining(), "");
    }

    #[test]
    pub fn clips() {
        let clip = Clip::from("four score and seven years ago");
        let mut searcher = clip.search();
        searcher.goto_pattern_start("and");
        let start = searcher.start_clip();
        searcher.goto_pattern_end("years");
        assert_eq!(searcher.end_clip(start).as_str(), "and seven years");
        searcher.goto_end();
        assert_eq!(searcher.end_clip(start).as_str(), "and seven years ago");
    }

    #[test]
    pub fn tri_split() {
        let clip = Clip::from("four score and seven years ago");
        let mut searcher = clip.search();
        searcher.goto_pattern_start("and");
        let start = searcher.start_clip();
        searcher.goto_pattern_end("years");
        let (clipped, (before, after)) = searcher.tri_split(start);
        assert_eq!(clipped.as_str(), "and seven years");
        assert_eq!(before.as_str(), "four score ");
        assert_eq!(after.as_str(), " ago");
    }

    #[test]
    pub fn delimiters() {
        let clip = Clip::from("{ { goal } } goal { }");
        let mut searcher = clip.search();
        searcher.skip_blocks(&[("{", "}")], "goal");
        assert_eq!(searcher.remaining(), "goal { }");

        let clip = Clip::from("{ ( goal ) } ( ( ( {()}))) goal { }");
        let mut searcher = clip.search();
        searcher.skip_blocks(&[("{", "}"), ("(", ")")], "goal");
        assert_eq!(searcher.remaining(), "goal { }");

        let clip = Clip::from("begin start hello start end end there end");
        let mut searcher = clip.search();
        searcher.goto_pattern_end("begin");
        searcher.skip_blocks(&[("start", "end")], "end");
        assert_eq!(searcher.remaining(), "end");

        let clip = Clip::from("start if elseif blahend endblah end end trailing");
        let mut searcher = clip.search();
        searcher.skip_blocks(&[("start", "end"), ("if", "end")], "trailing");
        assert_eq!(searcher.remaining(), "trailing");
    }

    #[test]
    pub fn pos() {
        let clip = Clip::from("hello\nhi\nlast line");
        let pos = clip.search().goto_pattern_end("hi").get_current_pos();
        assert_eq!(pos, Position::new(2, 3));
        let pos = clip
            .search()
            .goto_pattern_start("hello")
            .goto_pattern_start("last line")
            .get_current_pos();
        assert_eq!(pos, Position::new(3, 1));
    }
}
