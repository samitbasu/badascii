use roughr::{
    PathSegment,
    core::{Drawable, Options},
};

use crate::{analyze::get_wires, tc::TextCoordinate, text_buffer::TextBuffer};

/// Describes the parameters of the render from a text buffer
/// to the target (usually SVG).  You can control the `width`
/// and `height` of the virtual canvas, as well as the `x0`, `y0`
/// for the origin of the rendering on that canvas.
pub struct RenderJob {
    pub width: f32,
    pub height: f32,
    pub text: TextBuffer,
    pub options: roughr::core::Options,
    pub x0: f32,
    pub y0: f32,
}

impl RenderJob {
    /// Create a rendering job that uses rough lines for
    /// the drawing to give it a more informal look.
    pub fn rough(text: TextBuffer) -> Self {
        let width = (text.size().num_cols * 10) as f32;
        let height = (text.size().num_rows * 15) as f32;
        let options = Options::default();
        Self {
            width,
            height,
            text,
            options,
            x0: 0.0,
            y0: 0.0,
        }
    }
    /// Put on that suit and tie!  Time for a formal look.
    /// Only clean straight lines here.
    pub fn formal(text: TextBuffer) -> Self {
        let width = (text.size().num_cols * 10) as f32;
        let height = (text.size().num_rows * 15) as f32;
        let options = Options {
            disable_multi_stroke: Some(true),
            max_randomness_offset: Some(0.0),
            roughness: Some(0.0),
            ..Options::default()
        };
        Self {
            width,
            height,
            text,
            options,
            x0: 0.0,
            y0: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
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
    fn render_wire_end(&self, ch: char, pos: TextCoordinate) -> Vec<PathSegment> {
        let delta_x = self.width / self.text.size().num_cols as f32;
        let delta_y = self.height / self.text.size().num_rows as f32;
        let pos_map = |pos: TextCoordinate| {
            vec2(self.x0, self.y0)
                + vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y)
                + vec2(0.5 * delta_x, 0.5 * delta_y)
        };
        let p0 = pos_map(pos);
        match ch {
            //  *  \
            //  *  x  *
            //  *  /
            '>' => vec![
                move_to(p0 + vec2(0.0, -0.3 * delta_y)),
                line_to(p0 + vec2(1.0 * delta_x, 0.0)),
                line_to(p0 + vec2(0.0, 0.3 * delta_y)),
                close_path(),
            ],
            '<' => vec![
                move_to(p0 + vec2(0.0 * delta_x, -0.3 * delta_y)),
                line_to(p0 + vec2(-1.0 * delta_x, 0.0)),
                line_to(p0 + vec2(0.0 * delta_x, 0.3 * delta_y)),
                close_path(),
            ],
            'v' => vec![
                move_to(p0 + vec2(-0.5 * delta_x, 0.0)),
                line_to(p0 + vec2(0.0, 1.0 * delta_y)),
                line_to(p0 + vec2(0.5 * delta_x, 0.0)),
                close_path(),
            ],
            '^' => vec![
                move_to(p0 + vec2(-0.5 * delta_x, 0.0)),
                line_to(p0 + vec2(0.0, -1.0 * delta_y)),
                line_to(p0 + vec2(0.5 * delta_x, 0.0)),
                close_path(),
            ],
            _ => Vec::default(),
        }
    }

    pub fn invoke(&self) -> (TextBuffer, Vec<Drawable<f32>>) {
        let delta_x = self.width / self.text.size().num_cols as f32;
        let delta_y = self.height / self.text.size().num_rows as f32;
        let mut labels = self.text.clone();
        let pos_map = |pos: TextCoordinate| {
            vec2(self.x0, self.y0)
                + vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y)
                + vec2(0.5 * delta_x, 0.5 * delta_y)
        };
        let wires = get_wires(&labels);
        let generator = roughr::generator::Generator::default();
        let options = self.options.clone();
        let options = Some(options);
        let mut drawables = vec![];
        // Convert the wires into a list of Path Segments
        let mut path_segments: Vec<PathSegment> = wires
            .iter()
            .flat_map(|wire| {
                let p0 = pos_map(wire.start);
                let p1 = pos_map(wire.end);
                [move_to(p0), line_to(p1)]
            })
            .collect();
        for segment in &wires {
            for pt in segment.iter() {
                labels.set_text(&pt, None);
            }
        }
        // Draw end things
        for segment in wires {
            let pos = segment.start;
            if let Some(ch) = self.text.get(pos) {
                path_segments.extend(self.render_wire_end(ch, pos));
                labels.set_text(&pos, None);
            }
            let pos = segment.end;
            if let Some(ch) = self.text.get(pos) {
                path_segments.extend(self.render_wire_end(ch, pos));
                labels.set_text(&pos, None);
            }
        }
        let ops = generator.path_from_segments(path_segments, &options);
        drawables.push(ops);
        (labels, drawables)
    }
}
