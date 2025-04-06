use egui::Pos2;
use egui::{Rect, pos2, vec2};
use piet::RenderContext;
use piet::kurbo;
use roughr::core::{Drawable, OpSetType, OpType};

use crate::{
    analyze::get_wires,
    roughr_egui::{line_to, move_to},
    tc::TextCoordinate,
    text_buffer::TextBuffer,
};

pub struct RenderJob {
    pub width: f32,
    pub height: f32,
    pub num_cols: u32,
    pub num_rows: u32,
    pub labels: TextBuffer,
    pub options: roughr::core::Options,
}

fn kurbify(p: Pos2) -> kurbo::Point {
    kurbo::Point::new(p.x as f64, p.y as f64)
}

pub fn stroke_opset(ops: Drawable<f32>, painter: &mut piet_svg::RenderContext) {
    for op_set in ops.sets {
        if op_set.op_set_type != OpSetType::Path {
            continue;
        }
        let mut pos = pos2(0.0, 0.0);
        for op in op_set.ops {
            match op.op {
                OpType::Move => {
                    pos = pos2(op.data[0], op.data[1]);
                }
                OpType::LineTo => {
                    let new_pos = pos2(op.data[0], op.data[1]);
                    painter.stroke(
                        kurbo::Line::new(kurbify(pos), kurbify(new_pos)),
                        &piet::Color::GREEN,
                        1.0,
                    );
                    pos = new_pos;
                }
                OpType::BCurveTo => {
                    let cp1 = pos2(op.data[0], op.data[1]);
                    let cp2 = pos2(op.data[2], op.data[3]);
                    let end = pos2(op.data[4], op.data[5]);
                    painter.stroke(
                        kurbo::CubicBez::new(
                            kurbify(pos),
                            kurbify(cp1),
                            kurbify(cp2),
                            kurbify(end),
                        ),
                        &piet::Color::GREEN,
                        1.0,
                    );
                    pos = end;
                }
            }
        }
    }
}

pub fn render(job: RenderJob) -> String {
    let mut context =
        piet_svg::RenderContext::new(kurbo::Size::new(job.width.into(), job.height.into()));
    let delta_x = job.width / job.num_cols as f32;
    let delta_y = job.height / job.num_rows as f32;
    let mut labels = job.labels;
    let pos_map = |pos: TextCoordinate| {
        (vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y) + vec2(0.5 * delta_x, 0.5 * delta_y))
            .to_pos2()
    };
    let wires = get_wires(&labels);
    let generator = roughr::generator::Generator::default();
    let options = Some(job.options);
    for wire in wires {
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
        stroke_opset(ops, &mut context);
        // Draw end things
        /*         for segment in wire.segments {
                   let pos = segment.start;
                   if let Some(ch) = self.text.get(pos) {
                       self.render_wire_end(ch, canvas, pos, painter);
                       labels.set_text(&pos, None);
                   }
                   let pos = segment.end;
                   if let Some(ch) = self.text.get(pos) {
                       self.render_wire_end(ch, canvas, pos, painter);
                       labels.set_text(&pos, None);
                   }
               }
        */
    }
    /*     let text_size = delta_x.min(delta_y) * TEXT_SCALE_FACTOR;
    let monospace = FontId::monospace(text_size);
    for (coord, ch) in labels.iter() {
        let center = self.map_text_coordinate_to_cell_center(canvas, &coord);
        painter.text(
            center,
            Align2::CENTER_CENTER,
            ch,
            monospace.clone(),
            Color32::LIGHT_GREEN,
        );
    } */
    context.finish();
    context.display().to_string()
}
