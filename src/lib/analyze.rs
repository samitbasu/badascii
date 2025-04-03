use egui::ahash::{HashMap, HashSet};

use crate::lib::{rect::Rectangle, tc::TextCoordinate, text_buffer::TextBuffer};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct LineSegment {
    pub start: TextCoordinate,
    pub end: TextCoordinate,
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

pub fn get_rectangles(tb: &TextBuffer) -> HashSet<Rectangle> {
    let horz_segments = get_horizontal_line_segments(tb);
    let vert_segments = get_vertical_line_segments(tb);
    let mut corner_map = HashMap::<TextCoordinate, HashSet<DirectedLine>>::default();
    for (corner, ls) in horz_segments
        .iter()
        .flat_map(|ls| {
            [
                (ls.start, DirectedLine::Horiz(*ls)),
                (ls.end, DirectedLine::Horiz(*ls)),
            ]
        })
        .chain(vert_segments.iter().flat_map(|ls| {
            [
                (ls.start, DirectedLine::Vert(*ls)),
                (ls.end, DirectedLine::Vert(*ls)),
            ]
        }))
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
