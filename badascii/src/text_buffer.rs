use crate::{rect::Rectangle, tc::TextCoordinate};

pub struct Size {
    pub num_rows: u32,
    pub num_cols: u32,
}

#[derive(Clone, Debug, Hash)]
pub struct TextBuffer {
    buffer: Box<[Option<char>]>,
    num_rows: u32,
    num_cols: u32,
}

impl std::fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        for row in 0..self.num_rows {
            for col in 0..self.num_cols {
                f.write_char(self.buffer[(row * self.num_cols + col) as usize].unwrap_or(' '))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl TextBuffer {
    pub fn new(rows: u32, cols: u32) -> Self {
        Self {
            buffer: vec![None; (cols * rows) as usize].into_boxed_slice(),
            num_rows: rows,
            num_cols: cols,
        }
    }
    pub fn with_text(text: &str) -> Self {
        let num_cols = text.split('\n').map(|x| x.len()).max().unwrap_or(80) as u32;
        let num_rows = text.split('\n').count() as u32;
        let mut me = Self::new(num_rows, num_cols);
        me.paste(text, TextCoordinate { x: 0, y: 0 });
        me
    }
    pub fn size(&self) -> Size {
        Size {
            num_cols: self.num_cols,
            num_rows: self.num_rows,
        }
    }
    pub fn set_text(&mut self, pos: &TextCoordinate, ch: Option<char>) {
        let ch = if ch == Some(' ') { None } else { ch };
        if (0..self.num_cols).contains(&pos.x) && (0..self.num_rows).contains(&pos.y) {
            self.buffer[(pos.x + pos.y * self.num_cols) as usize] = ch;
        }
    }
    pub fn merge_text(&mut self, pos: &TextCoordinate, ch: Option<char>) {
        if let Some(ch) = ch {
            self.set_text(pos, Some(ch));
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (TextCoordinate, char)> {
        self.buffer.iter().enumerate().filter_map(|(ndx, c)| {
            if let Some(c) = c {
                let row = ndx as u32 / self.num_cols;
                let col = ndx as u32 % self.num_cols;
                Some((TextCoordinate { x: col, y: row }, *c))
            } else {
                None
            }
        })
    }
    pub fn iter_vert(&self) -> impl Iterator<Item = (TextCoordinate, char)> {
        (0..self.num_cols).flat_map(move |col| {
            (0..self.num_rows).flat_map(move |row| {
                self.buffer[(col + row * self.num_cols) as usize]
                    .map(|c| (TextCoordinate { x: col, y: row }, c))
            })
        })
    }
    // Iterate in diagonal slices
    //   1 4 6
    //   7 2 5
    //   9 8 3
    pub fn iter_diag_down_right(&self) -> impl Iterator<Item = (TextCoordinate, char)> {
        let first_col = (0..self.num_rows).map(|r| TextCoordinate { x: 0, y: r });
        let first_row = (1..self.num_cols).map(|c| TextCoordinate { x: c, y: 0 });
        let start_pos = first_col.chain(first_row);
        start_pos
            .flat_map(|s| {
                (0..)
                    .map(move |offset| TextCoordinate {
                        x: s.x + offset,
                        y: s.y + offset,
                    })
                    .take_while(|p| self.contains(p))
            })
            .flat_map(|p| self.get(p).map(|c| (p, c)))
    }
    pub fn iter_diag_up_right(&self) -> impl Iterator<Item = (TextCoordinate, char)> {
        let first_col = (0..self.num_rows).map(|r| TextCoordinate { x: 0, y: r });
        let last_row = (1..self.num_cols).map(|c| TextCoordinate {
            x: c,
            y: self.num_rows - 1,
        });
        let start_pos = first_col.chain(last_row);
        start_pos
            .flat_map(|s| {
                (0..=s.y)
                    .map(move |offset| TextCoordinate {
                        x: s.x + offset,
                        y: s.y - offset,
                    })
                    .take_while(|p| self.contains(p))
            })
            .flat_map(|p| self.get(p).map(|c| (p, c)))
    }
    fn contains(&self, tc: &TextCoordinate) -> bool {
        (0..self.num_rows).contains(&tc.y) && (0..self.num_cols).contains(&tc.x)
    }
    pub fn words(&self) -> impl Iterator<Item = (TextCoordinate, String)> {
        let mut prev_location: Option<TextCoordinate> = None;
        let mut start_location: Option<TextCoordinate> = None;
        let mut buffer = String::new();
        let mut iter = self.iter().fuse();
        std::iter::from_fn(move || {
            loop {
                // Get the next character
                if let Some((pos, ch)) = iter.next() {
                    // Check to see if it is adjacent to prev_location
                    if let Some(prev) = prev_location {
                        // Check if pos is adjacent to pos
                        if (pos.y == prev.y) && (prev.x + 1 == pos.x) {
                            buffer.push(ch);
                            prev_location = Some(pos);
                        } else {
                            prev_location = Some(pos);
                            let old_start = start_location.take();
                            start_location = Some(pos);
                            let to_ret = std::mem::take(&mut buffer);
                            buffer.push(ch);
                            return old_start.map(|x| (x, to_ret));
                        }
                    } else {
                        // First character... Stash it
                        prev_location = Some(pos);
                        start_location = Some(pos);
                        buffer.push(ch);
                    }
                } else if !buffer.is_empty() {
                    return start_location.map(|l| (l, std::mem::take(&mut buffer)));
                } else {
                    return None;
                }
            }
        })
    }
    pub fn clear_rectangle(&mut self, selection: Rectangle) {
        for pos in selection.iter_interior() {
            self.set_text(&pos, None);
        }
    }

    pub fn get(&self, pos: TextCoordinate) -> Option<char> {
        if (0..self.num_cols).contains(&pos.x) && (0..self.num_rows).contains(&pos.y) {
            self.buffer[(pos.x + pos.y * self.num_cols) as usize]
        } else {
            None
        }
    }

    pub fn clear_all(&mut self) {
        self.buffer.fill(None)
    }

    pub fn paste(&mut self, initial_text: &str, pos: TextCoordinate) -> Rectangle {
        let corner_1 = pos;
        let mut corner_2 = corner_1;
        for (row, line) in initial_text.lines().enumerate() {
            for (col, char) in line.chars().enumerate() {
                let pos = TextCoordinate {
                    x: pos.x + col as u32,
                    y: pos.y + row as u32,
                };
                corner_2.x = corner_2.x.max(pos.x);
                corner_2.y = corner_2.y.max(pos.y);
                self.set_text(&pos, Some(char))
            }
        }
        Rectangle { corner_1, corner_2 }
    }
    pub fn window(&self, rect: &Rectangle) -> TextBuffer {
        let mut out_buffer = TextBuffer::new(rect.height(), rect.width());
        let min_x = rect.left();
        let min_y = rect.top();
        for row in 0..rect.height() {
            for col in 0..rect.width() {
                out_buffer.set_text(
                    &TextCoordinate { x: col, y: row },
                    self.get(TextCoordinate {
                        x: min_x + col,
                        y: min_y + row,
                    }),
                )
            }
        }
        out_buffer
    }

    pub fn render(&self) -> String {
        let rows = self.buffer.chunks(self.num_cols as usize);
        let t = rows.flat_map(|x| {
            x.iter()
                .map(|c| c.unwrap_or(' '))
                .chain(std::iter::once('\n'))
        });
        let buf: String = t.collect();
        let buf = buf
            .split('\n')
            .map(|x| x.trim_ascii_end())
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();
        buf.join("\n")
    }

    pub fn resize(&self, resize: Size) -> TextBuffer {
        let mut output = TextBuffer::new(resize.num_rows, resize.num_cols);
        for row in 0..resize.num_rows {
            for col in 0..resize.num_cols {
                let tc = TextCoordinate { x: col, y: row };
                output.set_text(&tc, self.get(tc));
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_trailing_whitespace_trimmed_on_render() {
        let test_text = "
     +--+     
     |  |   
     +--+   
        ";
        let mut tb = TextBuffer::new(10, 20);
        tb.paste(test_text, TextCoordinate { x: 0, y: 1 });
        let render = tb.render();
        assert_eq!(
            render,
            "     +--+
     |  |
     +--+"
        );
    }

    #[test]
    fn test_diag_down_right_iterator() {
        //  123  159487263
        //  456
        //  789
        let test_text = "123\n456\n789\n";
        let tb = TextBuffer::with_text(test_text);
        let iter = tb.iter_diag_down_right().map(|x| x.1).collect::<String>();
        let expect = expect!["159487263"];
        expect.assert_eq(&iter);
    }

    #[test]
    fn test_diag_down_left_iterator() {
        //  1234    142753869
        //  5678    1 52 963 074 18 2
        //  9012
        let test_text = "1234\n5678\n9012\n";
        let tb = TextBuffer::with_text(test_text);
        let iter = tb.iter_diag_up_right().map(|x| x.1).collect::<String>();
        let expect = expect!["152963074182"];
        expect.assert_eq(&iter);
    }

    #[test]
    fn test_word_iterator() {
        let test_text = "
a bad 
    ay  at_the office        
        ";
        let mut tb = TextBuffer::new(5, 30);
        tb.paste(test_text, TextCoordinate { x: 1, y: 1 });
        let words = tb.words().collect::<Vec<_>>();
        let expect = expect![[r#"
            [
                (
                    TextCoordinate {
                        x: 1,
                        y: 2,
                    },
                    "a",
                ),
                (
                    TextCoordinate {
                        x: 3,
                        y: 2,
                    },
                    "bad",
                ),
                (
                    TextCoordinate {
                        x: 5,
                        y: 3,
                    },
                    "ay",
                ),
                (
                    TextCoordinate {
                        x: 9,
                        y: 3,
                    },
                    "at_the",
                ),
                (
                    TextCoordinate {
                        x: 16,
                        y: 3,
                    },
                    "office",
                ),
            ]
        "#]];
        expect.assert_debug_eq(&words);
    }
}
