use crate::{Resize, rect::Rectangle, tc::TextCoordinate};

#[derive(Clone, Debug)]
pub struct TextBuffer {
    buffer: Box<[Option<char>]>,
    num_rows: u32,
    num_cols: u32,
}

impl TextBuffer {
    pub fn new(rows: u32, cols: u32) -> Self {
        Self {
            buffer: vec![None; (cols * rows) as usize].into_boxed_slice(),
            num_rows: rows,
            num_cols: cols,
        }
    }
    pub fn set_text(&mut self, pos: &TextCoordinate, ch: Option<char>) {
        let ch = if ch == Some(' ') { None } else { ch };
        if (0..self.num_cols).contains(&pos.x) && (0..self.num_rows).contains(&pos.y) {
            self.buffer[(pos.x + pos.y * self.num_cols) as usize] = ch;
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
        t.collect()
    }

    pub fn resize(&self, resize: Resize) -> TextBuffer {
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
