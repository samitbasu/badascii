use rand::{RngCore, SeedableRng, rngs::StdRng};
use roughr::{PathSegment, core::Drawable};

use crate::{analyze::get_wires, tc::TextCoordinate, text_buffer::TextBuffer};

pub struct RenderJob {
    pub width: f32,
    pub height: f32,
    pub num_cols: u32,
    pub num_rows: u32,
    pub text: TextBuffer,
    pub options: roughr::core::Options,
    pub x0: f32,
    pub y0: f32,
}

#[derive(Copy, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

fn move_to(p: Vec2) -> PathSegment {
    PathSegment::MoveTo {
        abs: true,
        x: p.x as f64,
        y: p.y as f64,
    }
}

fn line_to(p: Vec2) -> PathSegment {
    PathSegment::LineTo {
        abs: true,
        x: p.x as f64,
        y: p.y as f64,
    }
}

fn close_path() -> PathSegment {
    PathSegment::ClosePath { abs: true }
}

impl RenderJob {
    fn render_wire_end(&self, ch: char, pos: TextCoordinate) -> Option<Drawable<f32>> {
        let delta_x = self.width / self.num_cols as f32;
        let delta_y = self.height / self.num_rows as f32;
        let pos_map = |pos: TextCoordinate| {
            vec2(self.x0, self.y0)
                + vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y)
                + vec2(0.5 * delta_x, 0.5 * delta_y)
        };
        let generator = roughr::generator::Generator::default();
        let options = Some(self.options.clone());
        let p0 = pos_map(pos);
        match ch {
            //  *  \
            //  *  x  *
            //  *  /
            '>' => Some(generator.path_from_segments(
                vec![
                    move_to(p0 + vec2(0.0, -0.3 * delta_y)),
                    line_to(p0 + vec2(1.0 * delta_x, 0.0)),
                    line_to(p0 + vec2(0.0, 0.3 * delta_y)),
                    close_path(),
                ],
                &options,
            )),
            '<' => Some(generator.path_from_segments(
                vec![
                    move_to(p0 + vec2(0.5 * delta_x, -0.2 * delta_y)),
                    line_to(p0 + vec2(-1.0 * delta_x, 0.0)),
                    line_to(p0 + vec2(0.5 * delta_x, 0.2 * delta_y)),
                    close_path(),
                ],
                &options,
            )),
            'v' => Some(generator.path_from_segments(
                vec![
                    move_to(p0 + vec2(-0.5 * delta_x, 0.0)),
                    line_to(p0 + vec2(0.0, 1.0 * delta_y)),
                    line_to(p0 + vec2(0.5 * delta_x, 0.0)),
                    close_path(),
                ],
                &options,
            )),
            '^' => Some(generator.path_from_segments(
                vec![
                    move_to(p0 + vec2(-0.5 * delta_x, 0.0)),
                    line_to(p0 + vec2(0.0, -1.0 * delta_y)),
                    line_to(p0 + vec2(0.5 * delta_x, 0.0)),
                    close_path(),
                ],
                &options,
            )),
            'o' => Some(generator.circle(p0.x, p0.y, delta_x, &options)),
            _ => None,
        }
    }

    pub fn invoke(&self) -> (TextBuffer, Vec<Drawable<f32>>) {
        let delta_x = self.width / self.num_cols as f32;
        let delta_y = self.height / self.num_rows as f32;
        let mut labels = self.text.clone();
        let pos_map = |pos: TextCoordinate| {
            vec2(self.x0, self.y0)
                + vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y)
                + vec2(0.5 * delta_x, 0.5 * delta_y)
        };
        let wires = get_wires(&labels);
        let generator = roughr::generator::Generator::default();
        let options = self.options.clone();
        // The RNG does not really work.  Each time we draw something,
        // the options struct is cloned which means that each wire is
        // drawn with the same RNG state at the beginning.
        let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF);
        let mut drawables = vec![];
        for wire in wires {
            let mut options = options.clone();
            options.randomizer = Some(StdRng::seed_from_u64(rng.next_u64()));
            let options = Some(options);
            let segments = wire
                .segments
                .iter()
                .flat_map(|ls| {
                    let p0 = pos_map(ls.start);
                    let p1 = pos_map(ls.end);
                    [move_to(p0), line_to(p1)]
                })
                .collect();
            for segment in &wire.segments {
                for pt in segment.iter() {
                    labels.set_text(&pt, None);
                }
            }
            let ops = generator.path_from_segments(segments, &options);
            drawables.push(ops);
            // Draw end things
            for segment in wire.segments {
                let pos = segment.start;
                if let Some(ch) = self.text.get(pos) {
                    drawables.extend(self.render_wire_end(ch, pos));
                    labels.set_text(&pos, None);
                }
                let pos = segment.end;
                if let Some(ch) = self.text.get(pos) {
                    drawables.extend(self.render_wire_end(ch, pos));
                    labels.set_text(&pos, None);
                }
            }
        }
        (labels, drawables)
    }
}
