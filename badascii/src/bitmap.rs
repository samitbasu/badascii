use std::{io::Write, sync::Arc};

use rasterize::{
    ActiveEdgeRasterizer, BBox, Image, LinColor, LineCap, LineJoin, Path, Scene, StrokeStyle,
    Transform,
};
use roughr::core::{Drawable, OpSetType, OpType};

use crate::{RenderJob, render::vec2, tc::TextCoordinate};

type Error = Box<dyn std::error::Error>;

pub fn stroke_opset(ops: Drawable<f32>, color: &str) -> Option<Scene> {
    let mut scenes = vec![];
    for op_set in ops.sets {
        if op_set.op_set_type != OpSetType::Path {
            continue;
        }
        let mut path = rasterize::PathBuilder::new();
        for op in op_set.ops {
            match op.op {
                OpType::Move => {
                    path.move_to((op.data[0] as f64, op.data[1] as f64));
                }
                OpType::LineTo => {
                    path.line_to((op.data[0] as f64, op.data[1] as f64));
                }
                OpType::BCurveTo => {
                    path.cubic_to(
                        (op.data[0] as f64, op.data[1] as f64),
                        (op.data[2] as f64, op.data[3] as f64),
                        (op.data[4] as f64, op.data[5] as f64),
                    );
                }
            }
        }
        scenes.push(Scene::stroke(
            path.build().into(),
            Arc::new(color.parse::<LinColor>().ok()?),
            StrokeStyle {
                width: 1.0,
                line_join: LineJoin::default(),
                line_cap: LineCap::Round,
            },
        ));
    }
    Some(Scene::group(scenes))
}

pub fn render(w: impl Write, job: &RenderJob, color: &str, background: &str) -> Result<(), Error> {
    let delta_x = job.width / job.text.size().num_cols as f32;
    let delta_y = job.height / job.text.size().num_rows as f32;
    let (labels, drawables) = job.invoke();
    let pos_map = |pos: TextCoordinate| {
        vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y) + vec2(0.5 * delta_x, 0.5 * delta_y)
    };
    let elements = drawables
        .into_iter()
        .flat_map(|op| stroke_opset(op, color))
        .collect::<Vec<_>>();
    let background = background.parse::<LinColor>().ok();
    let scene = Scene::group(elements);
    let image = scene.render(
        &ActiveEdgeRasterizer::default(),
        Transform::identity(),
        Some(BBox::new((0.0, 0.0), (job.width as f64, job.height as f64))),
        background,
    );
    image.write_png(w)?;
    Ok(())
    /*     let text_size = delta_x.min(delta_y) * 1.6;
       for (coord, word) in labels.iter() {
           let center = pos_map(coord);
           let text = svg::node::element::Text::new(word)
               .set("x", center.x)
               .set("y", center.y)
               .set("font-family", "monospace")
               .set("font-size", text_size)
               .set("text-anchor", "middle")
               .set("dominant-baseline", "middle")
               .set("fill", color);
           context = context.add(text);
       }
       context.to_string()
    */
}

#[cfg(test)]
mod tests {
    use crate::TextBuffer;

    use super::*;

    #[test]
    fn test_startup_screen() {
        let tb = TextBuffer::with_text(include_str!("startup_screen.txt"));
        let mut job = RenderJob::rough(tb);
        job.height *= 2.0;
        job.width *= 2.0;
        let path = std::path::Path::new(r"image.png");
        let file = std::fs::File::create(path).unwrap();
        let w = std::io::BufWriter::new(file);
        render(w, &job, "#FFFFFF", "#000000").unwrap();
    }
}
