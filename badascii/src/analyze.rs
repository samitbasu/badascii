use crate::{tc::TextCoordinate, text_buffer::TextBuffer};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct LineSegment {
    pub start: TextCoordinate,
    pub end: TextCoordinate,
}

#[derive(PartialEq, Eq)]
enum Kind {
    Horiz,
    Vert,
    DownSlant,
    UpSlant,
}

impl LineSegment {
    pub fn id(&self) -> u32 {
        let sx = self.start.x & 0xFF;
        let sy = self.start.y & 0xFF;
        let ex = self.end.x & 0xFF;
        let ey = self.end.y & 0xFF;
        (ey << 24) | (ex << 16) | (sy << 8) | (sx)
    }
    fn kind(&self) -> Kind {
        let del_x = (self.end.x as i32) - (self.start.x as i32);
        let del_y = (self.end.y as i32) - (self.start.y as i32);
        if del_y == 0 {
            Kind::Horiz
        } else if del_x == 0 {
            Kind::Vert
        } else if del_x >= 0 && del_y >= 0 {
            Kind::DownSlant
        } else {
            Kind::UpSlant
        }
    }
    //
    //   *                       *
    //    *    dx>0, dy>0       *   dx>0, dy<0
    //     *                   *
    pub fn iter(&self) -> impl Iterator<Item = TextCoordinate> {
        let kind = self.kind();
        let iter_range = match kind {
            Kind::UpSlant | Kind::DownSlant | Kind::Horiz => {
                self.end.x.saturating_sub(self.start.x)
            }
            Kind::Vert => self.end.y.saturating_sub(self.start.y),
        };
        let iter_range = 0..iter_range;
        let mk_point = move |p| match kind {
            Kind::Horiz => TextCoordinate {
                x: self.start.x + p,
                y: self.start.y,
            },
            Kind::Vert => TextCoordinate {
                x: self.start.x,
                y: self.start.y + p,
            },
            Kind::DownSlant => TextCoordinate {
                x: self.start.x + p,
                y: self.start.y + p,
            },
            Kind::UpSlant => TextCoordinate {
                x: self.start.x + p,
                y: self.start.y - p,
            },
        };
        iter_range.map(mk_point)
    }
    fn len(&self) -> u32 {
        let del_x = (self.end.x as i32 - self.start.x as i32).abs();
        let del_y = (self.end.y as i32 - self.start.y as i32).abs();
        del_x.max(del_y) as u32
    }
    fn is_colinear(&self, other: &LineSegment) -> bool {
        (self.kind() == other.kind())
            && ((self.start == other.start)
                || (self.end == other.start)
                || (self.end == other.end)
                || (self.start == other.end))
    }
    fn extend(&mut self, other: &LineSegment) {
        assert!(self.is_colinear(other));
        // Because the line segments are colinear,
        // we can compute the concatenated line segment
        // by taking the bounding "Rect", which will be degenerate.
        let Some(&min_x) = [self.start.x, self.end.x, other.start.x, other.end.x]
            .iter()
            .min()
        else {
            return;
        };
        let Some(&max_x) = [self.start.x, self.end.x, other.start.x, other.end.x]
            .iter()
            .max()
        else {
            return;
        };
        let Some(&min_y) = [self.start.y, self.end.y, other.start.y, other.end.y]
            .iter()
            .min()
        else {
            return;
        };
        let Some(&max_y) = [self.start.y, self.end.y, other.start.y, other.end.y]
            .iter()
            .max()
        else {
            return;
        };
        match self.kind() {
            Kind::Horiz | Kind::Vert | Kind::DownSlant => {
                self.start.x = min_x;
                self.start.y = min_y;
                self.end.x = max_x;
                self.end.y = max_y;
            }
            Kind::UpSlant => {
                self.start.x = min_x;
                self.start.y = max_y;
                self.end.x = max_x;
                self.end.y = min_y;
            }
        }
    }
}

#[derive(Debug)]
enum State {
    Blank,
    Tracking(LineSegment),
}

#[derive(PartialEq, Debug)]
enum Class {
    Term,
    Edge,
    End,
}

fn classify_horiz(ch: char) -> Option<Class> {
    match ch {
        '+' | '<' | '>' => Some(Class::Term),
        '-' => Some(Class::Edge),
        _ => None,
    }
}

fn classify_vert(ch: char) -> Option<Class> {
    match ch {
        '+' | '^' | 'v' => Some(Class::Term),
        '|' => Some(Class::Edge),
        _ => None,
    }
}

fn classify_diag_down_left(ch: char) -> Option<Class> {
    match ch {
        '+' => Some(Class::Term),
        '/' => Some(Class::Edge),
        _ => None,
    }
}

fn classify_diag_down_right(ch: char) -> Option<Class> {
    match ch {
        '+' => Some(Class::Term),
        '\\' => Some(Class::Edge),
        _ => None,
    }
}

fn merge_line_segment(segments: &mut Vec<LineSegment>, segment: LineSegment) {
    for candidate in segments.iter_mut() {
        if candidate.is_colinear(&segment) {
            candidate.extend(&segment);
            return;
        }
    }
    segments.push(segment);
}

fn merge_colinear(mut segments: Vec<LineSegment>) -> Vec<LineSegment> {
    let mut ret = vec![];
    let Some(segment) = segments.pop() else {
        return ret;
    };
    ret.push(segment);
    for segment in segments {
        merge_line_segment(&mut ret, segment);
    }
    ret
}

pub fn get_wires(tb: &TextBuffer) -> Vec<LineSegment> {
    let mut segments = get_horizontal_line_segments(tb);
    segments.extend(get_vertical_line_segments(tb));
    segments.extend(get_diag_up_right_segments(tb));
    segments.extend(get_diag_down_right_segments(tb));
    let mut segments = merge_colinear(segments);
    segments.sort_by_key(|l| l.id());
    segments
}

const EOB: (TextCoordinate, Class) = (
    TextCoordinate {
        x: 100_000,
        y: 100_000,
    },
    Class::End,
);

fn line_segment_finder<N>(
    vals: impl Iterator<Item = (TextCoordinate, Class)>,
    valid_next: N,
) -> Vec<LineSegment>
where
    N: Fn(&TextCoordinate, &TextCoordinate) -> bool,
{
    let mut state = State::Blank;
    let mut lines = vec![];
    for (pos, kind) in vals.chain(std::iter::once(EOB)) {
        match (state, pos, kind) {
            (State::Blank, pos, Class::Term) => {
                state = State::Tracking(LineSegment {
                    start: pos,
                    end: pos,
                })
            }
            (State::Tracking(track), pos, class) if valid_next(&track.end, &pos) => {
                // We have a new character along the track
                match class {
                    Class::Term => {
                        lines.push(LineSegment {
                            start: track.start,
                            end: pos,
                        });
                        state = State::Tracking(LineSegment {
                            start: pos,
                            end: pos,
                        })
                    }
                    Class::End => state = State::Blank,
                    Class::Edge => {
                        state = State::Tracking(LineSegment {
                            start: track.start,
                            end: pos,
                        });
                    }
                }
            }
            (State::Tracking(_track), pos, Class::Term) => {
                // We got a term, but it wasn't the next character.
                // So restart the tracking with this position
                state = State::Tracking(LineSegment {
                    start: pos,
                    end: pos,
                });
            }
            _ => {
                state = State::Blank;
            }
        }
    }
    lines.retain(|ls| ls.len() >= 1);
    lines
}

fn get_vertical_line_segments(tb: &TextBuffer) -> Vec<LineSegment> {
    line_segment_finder(
        tb.iter_vert()
            .filter_map(|(pos, ch)| classify_vert(ch).map(|k| (pos, k))),
        |track, candidate| track.x == candidate.x && track.y + 1 == candidate.y,
    )
}

fn get_horizontal_line_segments(tb: &TextBuffer) -> Vec<LineSegment> {
    line_segment_finder(
        tb.iter()
            .filter_map(|(pos, ch)| classify_horiz(ch).map(|k| (pos, k))),
        |track, candidate| track.y == candidate.y && track.x + 1 == candidate.x,
    )
}

fn get_diag_down_right_segments(tb: &TextBuffer) -> Vec<LineSegment> {
    line_segment_finder(
        tb.iter_diag_down_right()
            .filter_map(|(pos, ch)| classify_diag_down_right(ch).map(|k| (pos, k))),
        |track, candidate| track.y + 1 == candidate.y && track.x + 1 == candidate.x,
    )
}

fn get_diag_up_right_segments(tb: &TextBuffer) -> Vec<LineSegment> {
    line_segment_finder(
        tb.iter_diag_up_right()
            .filter_map(|(pos, ch)| classify_diag_down_left(ch).map(|k| (pos, k))),
        |track, candidate| track.y == candidate.y + 1 && track.x + 1 == candidate.x,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wires() {
        const CUP_EXAMPLE: &str = "
+-----+
      |
      |<-----o 
      |
+-----+
";
        let mut text_buffer = TextBuffer::new(20, 20);
        text_buffer.paste(CUP_EXAMPLE, TextCoordinate { x: 1, y: 1 });
        let wires = get_wires(&text_buffer);
        assert_eq!(wires.len(), 2);
    }

    #[test]
    fn test_vert_arrow() {
        const INITIAL_TEXT: &str = "
        
        +
        |
        v

        ";
        let mut buffer = TextBuffer::new(20, 20);
        buffer.paste(INITIAL_TEXT, TextCoordinate { x: 4, y: 4 });
        let wires = get_wires(&buffer);
        assert_eq!(wires.len(), 1);
    }

    #[test]
    fn test_colinear_wire() {
        const INITIAL_TEXT: &str = "
    +
    |
+---+---+        
    |
    +    
        ";
        let mut buffer = TextBuffer::new(20, 20);
        buffer.paste(INITIAL_TEXT, TextCoordinate { x: 2, y: 2 });
        let wires = get_wires(&buffer);
        assert_eq!(wires.len(), 2);
    }

    #[test]
    fn test_short_line_extraction() {
        const INITIAL_TEXT: &str = "
  +
  v        
        ";
        let mut buffer = TextBuffer::new(20, 20);
        buffer.paste(INITIAL_TEXT, TextCoordinate { x: 2, y: 2 });
        let wires = get_wires(&buffer);
        let expect = expect_test::expect![[r#"
            [
                LineSegment {
                    start: TextCoordinate {
                        x: 4,
                        y: 3,
                    },
                    end: TextCoordinate {
                        x: 4,
                        y: 4,
                    },
                },
            ]
        "#]];
        expect.assert_debug_eq(&wires);
    }
}
