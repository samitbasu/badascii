use std::collections::{HashMap, HashSet};

use crate::{tc::TextCoordinate, text_buffer::TextBuffer};

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

fn classify(ch: char) -> Option<Class> {
    match ch {
        '+' | '<' | '>' | '^' | 'v' => Some(Class::Term),
        '-' => Some(Class::HorizEdge),
        '|' => Some(Class::VertEdge),
        _ => None,
    }
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
    lines.retain(|ls| ls.len() >= 1);
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
}
