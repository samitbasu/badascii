use egui::ahash::{HashMap, HashSet};

use crate::{rect::Rectangle, tc::TextCoordinate, text_buffer::TextBuffer};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct LineSegment {
    pub start: TextCoordinate,
    pub end: TextCoordinate,
}

fn line_segment(start: TextCoordinate, end: TextCoordinate) -> LineSegment {
    LineSegment { start, end }
}

enum State {
    Blank,
    Tracking(LineSegment),
    BlankCheck(LineSegment),
}

#[derive(PartialEq)]
enum Class {
    Corner,
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
        '+' => Some(Class::Corner),
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
        if corner_map.contains_key(&corner_2) {
            ret.insert(Rectangle { corner_1, corner_2 }.normalize());
        }
    }
    ret
}

fn get_vertical_line_segments(tb: &TextBuffer) -> Vec<LineSegment> {
    let mut state = State::Blank;
    let mut lines = vec![];

    for (pos, kind) in tb
        .iter_vert()
        .filter_map(|(pos, ch)| classify(ch).map(|k| (pos, k)))
        .chain(std::iter::once((
            TextCoordinate {
                x: 100_000,
                y: 100_000,
            },
            Class::End,
        )))
    {
        match (state, pos, kind) {
            (State::Blank, pos, Class::Corner) => {
                state = State::Tracking(LineSegment {
                    start: pos,
                    end: pos,
                })
            }
            (State::Tracking(track), pos, Class::VertEdge)
                if (track.start.x == pos.x) && (track.end.y + 1 == pos.y) =>
            {
                state = State::Tracking(LineSegment {
                    start: track.start,
                    end: pos,
                })
            }
            (State::Tracking(track), pos, Class::Corner)
                if (track.start.x == pos.x) && (track.end.y + 1 == pos.y) =>
            {
                state = State::BlankCheck(LineSegment {
                    start: track.start,
                    end: pos,
                })
            }
            (State::BlankCheck(track), pos, any)
                if (track.end.x != pos.x) || (track.end.y + 1 != pos.y) =>
            {
                lines.push(track);
                if any == Class::Corner {
                    state = State::Tracking(LineSegment {
                        start: pos,
                        end: pos,
                    })
                } else {
                    state = State::Blank;
                }
            }
            _ => {
                state = State::Blank;
            }
        }
    }
    lines
}

fn get_horizontal_line_segments(tb: &TextBuffer) -> Vec<LineSegment> {
    let mut state = State::Blank;
    let mut lines = vec![];

    for (pos, kind) in tb
        .iter()
        .filter_map(|(pos, ch)| classify(ch).map(|k| (pos, k)))
        .chain(std::iter::once((
            TextCoordinate {
                x: 100_000,
                y: 100_000,
            },
            Class::End,
        )))
    {
        match (state, pos, kind) {
            (State::Blank, pos, Class::Corner) => {
                state = State::Tracking(LineSegment {
                    start: pos,
                    end: pos,
                })
            }
            (State::Tracking(track), pos, Class::HorizEdge)
                if (track.start.y == pos.y) && (track.end.x + 1 == pos.x) =>
            {
                state = State::Tracking(LineSegment {
                    start: track.start,
                    end: pos,
                })
            }
            (State::Tracking(track), pos, Class::Corner)
                if (track.start.y == pos.y) && (track.end.x + 1 == pos.x) =>
            {
                state = State::BlankCheck(LineSegment {
                    start: track.start,
                    end: pos,
                })
            }
            (State::BlankCheck(track), pos, any)
                if (track.end.y != pos.y) || (track.end.x + 1 != pos.x) =>
            {
                lines.push(track);
                if any == Class::Corner {
                    state = State::Tracking(LineSegment {
                        start: pos,
                        end: pos,
                    })
                } else {
                    state = State::Blank;
                }
            }
            _ => {
                state = State::Blank;
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
