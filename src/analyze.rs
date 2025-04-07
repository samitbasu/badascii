use egui::ahash::{HashMap, HashSet};

use crate::{rect::Rectangle, tc::TextCoordinate, text_buffer::TextBuffer};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct LineSegment {
    pub start: TextCoordinate,
    pub end: TextCoordinate,
}

impl LineSegment {
    pub fn id(&self) -> u32 {
        let sx = self.start.x & 0xFF;
        let sy = self.start.y & 0xFF;
        let ex = self.end.x & 0xFF;
        let ey = self.end.y & 0xFF;
        (ey << 24) | (ex << 16) | (sy << 8) | (sx)
    }
    pub fn iter(&self) -> impl Iterator<Item = TextCoordinate> {
        let self_is_horiz = self.start.y == self.end.y;
        let iter_range = if self_is_horiz {
            self.start.x..=self.end.x
        } else {
            self.start.y..=self.end.y
        };
        let mk_point = move |p| {
            if self_is_horiz {
                TextCoordinate {
                    x: p,
                    y: self.start.y,
                }
            } else {
                TextCoordinate {
                    x: self.start.x,
                    y: p,
                }
            }
        };
        iter_range.map(mk_point)
    }
    fn len(&self) -> u32 {
        let del_x = (self.end.x as i32 - self.start.x as i32).abs();
        let del_y = (self.end.y as i32 - self.start.y as i32).abs();
        del_x.max(del_y) as u32
    }
    fn is_colinear(&self, other: &LineSegment) -> bool {
        let self_is_horiz = self.start.y == self.end.y;
        let other_is_horiz = other.start.y == other.end.y;
        if self_is_horiz && other_is_horiz {
            (self.start.y == other.start.y)
                && (self.start.x == other.end.x
                    || self.start.x == other.start.x
                    || self.end.x == other.start.x
                    || self.end.x == other.end.x)
        } else if !self_is_horiz && !other_is_horiz {
            (self.start.x == other.start.x)
                && (self.start.y == other.end.y
                    || self.start.y == other.start.y
                    || self.end.y == other.start.y
                    || self.end.y == other.end.y)
        } else {
            false
        }
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
        self.start.x = min_x;
        self.start.y = min_y;
        self.end.x = max_x;
        self.end.y = max_y;
    }
}

fn line_segment(start: TextCoordinate, end: TextCoordinate) -> LineSegment {
    LineSegment { start, end }
}

#[derive(Debug)]
enum State {
    Blank,
    Tracking(LineSegment),
}

#[derive(PartialEq, Debug)]
enum Class {
    Term,
    HorizEdge,
    VertEdge,
    End,
}

#[derive(Debug, Eq, PartialEq, Hash)]
enum DirectedLine {
    Horiz(LineSegment),
    Vert(LineSegment),
}

fn classify(ch: char) -> Option<Class> {
    match ch {
        '+' | 'o' | '<' | '>' | '^' | 'v' => Some(Class::Term),
        '-' => Some(Class::HorizEdge),
        '|' => Some(Class::VertEdge),
        _ => None,
    }
}

// We apply the constraint that for horizontal lines,
// that the start is to the left of end
fn mk_horiz(ls: LineSegment) -> DirectedLine {
    assert_eq!(ls.end.y, ls.start.y);
    let y = ls.end.y;
    let min_x = ls.start.x.min(ls.end.x);
    let max_x = ls.start.x.max(ls.end.x);
    DirectedLine::Horiz(LineSegment {
        start: TextCoordinate { x: min_x, y },
        end: TextCoordinate { x: max_x, y },
    })
}

fn mk_vert(ls: LineSegment) -> DirectedLine {
    assert_eq!(ls.end.x, ls.start.x);
    let x = ls.end.x;
    let min_y = ls.start.y.min(ls.end.y);
    let max_y = ls.start.y.max(ls.end.y);
    DirectedLine::Vert(LineSegment {
        start: TextCoordinate { x, y: min_y },
        end: TextCoordinate { x, y: max_y },
    })
}

#[derive(Debug, Hash)]
pub struct Wire {
    pub segments: Vec<LineSegment>,
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

pub fn get_wires(tb: &TextBuffer) -> Vec<Wire> {
    let mut segments = get_horizontal_line_segments(tb);
    segments.extend(get_vertical_line_segments(tb));
    let mut corner_map = HashMap::<TextCoordinate, HashSet<LineSegment>>::default();
    for ls in segments.clone() {
        corner_map.entry(ls.start).or_default().insert(ls);
        corner_map.entry(ls.end).or_default().insert(ls);
    }
    let segments = merge_colinear(segments);
    let mut segments: HashSet<LineSegment> = segments.into_iter().collect();
    let mut wireset = vec![];
    loop {
        let mut wire = vec![];
        let Some(segment) = segments.iter().next().cloned() else {
            break;
        };
        segments.remove(&segment);
        corner_map
            .get_mut(&segment.start)
            .map(|t| t.remove(&segment));
        corner_map.get_mut(&segment.end).map(|t| t.remove(&segment));
        wire.push(segment);
        let mut end_points: HashSet<TextCoordinate> =
            [segment.start, segment.end].into_iter().collect();
        loop {
            let Some(attached) = end_points
                .iter()
                .find_map(|x| corner_map.get(x).and_then(|p| p.iter().next().cloned()))
            else {
                break;
            };
            end_points.insert(attached.start);
            end_points.insert(attached.end);
            wire.push(attached);
            segments.remove(&attached);
            corner_map
                .get_mut(&attached.start)
                .map(|t| t.remove(&attached));
            corner_map
                .get_mut(&attached.end)
                .map(|t| t.remove(&attached));
        }
        wire.sort_by_key(|a| a.id());
        wireset.push(Wire { segments: wire });
    }
    wireset.sort_by_key(|x| x.segments[0].id());
    wireset
}

pub fn get_rectangles(tb: &TextBuffer) -> HashSet<Rectangle> {
    let horz_segments = get_horizontal_line_segments(tb);
    let vert_segments = get_vertical_line_segments(tb);
    let mut corner_map = HashMap::<TextCoordinate, HashSet<DirectedLine>>::default();
    for (corner, ls) in horz_segments
        .iter()
        .flat_map(|ls| [(ls.start, mk_horiz(*ls)), (ls.end, mk_horiz(*ls))])
        .chain(
            vert_segments
                .iter()
                .flat_map(|ls| [(ls.start, mk_vert(*ls)), (ls.end, mk_vert(*ls))]),
        )
    {
        corner_map.entry(corner).or_default().insert(ls);
    }

    let mut ret = HashSet::default();
    for (&corner_1, edges) in corner_map.iter() {
        let Some(horiz) = edges.iter().find_map(|x| match x {
            DirectedLine::Horiz(line) => Some(*line),
            _ => None,
        }) else {
            continue;
        };
        let Some(vert) = edges.iter().find_map(|x| match x {
            DirectedLine::Vert(line) => Some(*line),
            _ => None,
        }) else {
            continue;
        };
        let opposite_x = if horiz.start.x == corner_1.x {
            horiz.end.x
        } else {
            horiz.start.x
        };
        let opposite_y = if vert.start.y == corner_1.y {
            vert.end.y
        } else {
            vert.start.y
        };
        let corner_2 = TextCoordinate {
            x: opposite_x,
            y: opposite_y,
        };
        if !corner_map.contains_key(&corner_2) {
            continue;
        }
        let candidate = Rectangle { corner_1, corner_2 }.normalize();
        let top = mk_horiz(line_segment(candidate.left_top(), candidate.right_top()));
        let right = mk_vert(line_segment(
            candidate.right_top(),
            candidate.right_bottom(),
        ));
        let left = mk_vert(line_segment(candidate.left_top(), candidate.left_bottom()));
        let bottom = mk_horiz(line_segment(
            candidate.left_bottom(),
            candidate.right_bottom(),
        ));
        let Some(top_left_edges) = corner_map.get(&candidate.left_top()) else {
            continue;
        };
        if !top_left_edges.contains(&top) || !top_left_edges.contains(&left) {
            continue;
        }
        let Some(bottom_right_edges) = corner_map.get(&candidate.right_bottom()) else {
            continue;
        };
        if !bottom_right_edges.contains(&bottom) || !bottom_right_edges.contains(&right) {
            continue;
        }
        let corner_1 = candidate.left_top();
        let corner_2 = candidate.right_bottom();
        let corner_3 = candidate.right_top();
        let corner_4 = candidate.left_bottom();
        // Check for empty corners
        let extended_spaces = [
            corner_1.up(),
            corner_1.left(),
            corner_2.down(),
            corner_2.right(),
            corner_3.right(),
            corner_3.up(),
            corner_4.left(),
            corner_4.down(),
        ];
        if extended_spaces.into_iter().any(|x| tb.get(x).is_some()) {
            continue;
        }
        // Check that corners are marked with '+'
        if ![corner_1, corner_2, corner_3, corner_4]
            .into_iter()
            .all(|x| tb.get(x) == Some('+'))
        {
            continue;
        }
        ret.insert(Rectangle { corner_1, corner_2 }.normalize());
    }
    ret
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
    edge: Class,
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
                    k if k == edge => {
                        state = State::Tracking(LineSegment {
                            start: track.start,
                            end: pos,
                        });
                    }
                    _ => state = State::Blank,
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
    lines.retain(|ls| ls.len() > 1);
    lines
}

fn get_vertical_line_segments(tb: &TextBuffer) -> Vec<LineSegment> {
    line_segment_finder(
        tb.iter_vert()
            .filter_map(|(pos, ch)| classify(ch).map(|k| (pos, k))),
        Class::VertEdge,
        |track, candidate| track.x == candidate.x && track.y + 1 == candidate.y,
    )
}

fn get_horizontal_line_segments(tb: &TextBuffer) -> Vec<LineSegment> {
    line_segment_finder(
        tb.iter()
            .filter_map(|(pos, ch)| classify(ch).map(|k| (pos, k))),
        Class::HorizEdge,
        |track, candidate| track.y == candidate.y && track.x + 1 == candidate.x,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rect() {
        const BASIC_EXAMPLE: &str = "
   +-----+
   |     |
   +-----+        
        ";
        let mut text_buffer = TextBuffer::new(20, 20);
        text_buffer.paste(BASIC_EXAMPLE, TextCoordinate { x: 1, y: 1 });
        let rects = get_rectangles(&text_buffer);
        assert!(rects.len() == 1);
    }

    #[test]
    fn test_extract_mis_hits() {
        const CUP_EXAMPLE: &str = "
+-----+
      |
      |
      |
+-----+
";
        let mut text_buffer = TextBuffer::new(20, 20);
        text_buffer.paste(CUP_EXAMPLE, TextCoordinate { x: 1, y: 1 });
        let rects = get_rectangles(&text_buffer);
        assert!(rects.is_empty());
    }

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
    fn test_empty_corners_checked() {
        const NONEMPTY_CORNER_EXAMPLE: &str = "
----+-----+
    |     |
    +-----+        
        ";
        let mut text_buffer = TextBuffer::new(20, 20);
        text_buffer.paste(NONEMPTY_CORNER_EXAMPLE, TextCoordinate { x: 1, y: 1 });
        let rects = get_rectangles(&text_buffer);
        assert!(rects.is_empty());
    }

    #[test]
    fn test_initial_diagram() {
        const INITIAL_TEXT: &str = "
     +---------------------+
     |                     |
    >| data           data |o
     |                     |
    o| full           next |>
     |                     |
    o| overflow  underflow |o   
     |                     |
     +---------------------+
";
        let mut buffer = TextBuffer::new(40, 40);
        buffer.paste(INITIAL_TEXT, TextCoordinate { x: 4, y: 4 });
        let rects = get_rectangles(&buffer);
        assert_eq!(rects.len(), 1);
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
}
