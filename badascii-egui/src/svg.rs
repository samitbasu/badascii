use egui::Pos2;
use egui::{pos2, vec2};
use roughr::core::{Drawable, OpSetType, OpType};

use crate::roughr_egui::close_path;
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

pub fn stroke_opset(ops: Drawable<f32>, mut painter: svg::Document) -> svg::Document {
    for op_set in ops.sets {
        if op_set.op_set_type != OpSetType::Path {
            continue;
        }
        let mut data = svg::node::element::path::Data::new();
        for op in op_set.ops {
            match op.op {
                OpType::Move => {
                    data = data.move_to(op.data);
                }
                OpType::LineTo => {
                    data = data.line_to(op.data);
                }
                OpType::BCurveTo => {
                    data = data.cubic_curve_to(op.data);
                }
            }
        }
        let path = svg::node::element::Path::new()
            .set("fill", "none")
            .set("stroke", "white")
            .set("stroke-width", 1)
            .set("d", data);
        painter = painter.add(path);
    }
    painter
}

fn render_wire_end(
    ch: char,
    job: &RenderJob,
    pos: TextCoordinate,
    painter: svg::Document,
) -> svg::Document {
    let top_left = pos2(0.0, 0.0);
    let delta_x = job.width / job.num_cols as f32;
    let delta_y = job.height / job.num_rows as f32;
    let pos_map = |pos: TextCoordinate| {
        top_left
            + vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y)
            + vec2(0.5 * delta_x, 0.5 * delta_y)
    };
    let generator = roughr::generator::Generator::default();
    let options = Some(job.options.clone());
    let p0 = pos_map(pos);
    let ops = match ch {
        //  *  \
        //  *  x  *
        //  *  /
        '>' => Some(generator.path_from_segments(
            vec![
                move_to(p0 + vec2(-0.5 * delta_x, -0.2 * delta_y)),
                line_to(p0 + vec2(0.5 * delta_x, 0.0)),
                line_to(p0 + vec2(-0.5 * delta_x, 0.2 * delta_y)),
                close_path(),
            ],
            &options,
        )),
        '<' => Some(generator.path_from_segments(
            vec![
                move_to(p0 + vec2(0.5 * delta_x, -0.2 * delta_y)),
                line_to(p0 + vec2(-0.5 * delta_x, 0.0)),
                line_to(p0 + vec2(0.5 * delta_x, 0.2 * delta_y)),
                close_path(),
            ],
            &options,
        )),
        'v' => Some(generator.path_from_segments(
            vec![
                move_to(p0 + vec2(-0.5 * delta_x, -0.2 * delta_y)),
                line_to(p0 + vec2(0.0, 0.2 * delta_y)),
                line_to(p0 + vec2(0.5 * delta_x, -0.2 * delta_y)),
                close_path(),
            ],
            &options,
        )),
        '^' => Some(generator.path_from_segments(
            vec![
                move_to(p0 + vec2(-delta_x, 0.2 * delta_y)),
                line_to(p0 + vec2(0.0, -0.2 * delta_y)),
                line_to(p0 + vec2(delta_x, 0.2 * delta_y)),
                close_path(),
            ],
            &options,
        )),
        'o' => Some(generator.circle(p0.x, p0.y, delta_x, &options)),
        _ => None,
    };
    let Some(ops) = ops else {
        return painter;
    };
    stroke_opset(ops, painter)
}

pub fn render(job: &RenderJob) -> String {
    let mut context = svg::Document::new().set("viewBox", (0.0, 0.0, job.width, job.height));
    let delta_x = job.width / job.num_cols as f32;
    let delta_y = job.height / job.num_rows as f32;
    let mut labels = job.labels.clone();
    let pos_map = |pos: TextCoordinate| {
        (vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y) + vec2(0.5 * delta_x, 0.5 * delta_y))
            .to_pos2()
    };
    let wires = get_wires(&labels);
    let generator = roughr::generator::Generator::default();
    let options = Some(job.options.clone());
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
        context = stroke_opset(ops, context);
        // Draw end things
        for segment in wire.segments {
            let pos = segment.start;
            if let Some(ch) = job.labels.get(pos) {
                context = render_wire_end(ch, job, pos, context);
                labels.set_text(&pos, None);
            }
            let pos = segment.end;
            if let Some(ch) = job.labels.get(pos) {
                context = render_wire_end(ch, job, pos, context);
                labels.set_text(&pos, None);
            }
        }
    }
    let text_size = delta_x.min(delta_y) * 1.6;
    /*

           let text = svg::node::element::Text::new(format!("{}", ndx * time_delta));
       document = document.add(
           svg::node::element::Line::new()
               .set("x1", x)
               .set("y1", 0)
               .set("x2", x)
               .set("y2", height)
               .set("stroke", "#333333")
               .set("stroke-width", 1.0),
       );

    */
    for (coord, word) in labels.iter() {
        let center = pos_map(coord);
        let text = svg::node::element::Text::new(word)
            .set("x", center.x)
            .set("y", center.y)
            .set("font-family", "monospace")
            .set("font-size", text_size)
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .set("fill", "#D4D4D4");
        context = context.add(text);
    }
    context.to_string()
}
