use std::sync::Arc;

use ab_glyph::ScaleFont;
use rasterize::{
    ActiveEdgeRasterizer, BBox, Color, Image, ImageMut, LinColor, LineCap, LineJoin, Scene,
    StrokeStyle, Transform,
};
use roughr::core::{Drawable, OpSetType, OpType};

use crate::{RenderJob, render::vec2, tc::TextCoordinate};

type Error = Box<dyn std::error::Error>;

pub fn stroke_opset(ops: Drawable<f32>, color: LinColor) -> Scene {
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
            Arc::new(color),
            StrokeStyle {
                width: 1.0,
                line_join: LineJoin::default(),
                line_cap: LineCap::Round,
            },
        ));
    }
    Scene::group(scenes)
}

pub fn render(
    job: &RenderJob,
    color: &str,
    background: &str,
) -> Result<rasterize::Layer<LinColor>, Error> {
    use ab_glyph::{Font, FontRef, Glyph, point};

    let font = FontRef::try_from_slice(include_bytes!("../font/Hack-Regular.ttf"))?;
    let color = color.parse::<LinColor>()?;
    let delta_x = job.width / job.text.size().num_cols as f32;
    let delta_y = job.height / job.text.size().num_rows as f32;
    let (labels, drawables) = job.invoke();
    let pos_map = |pos: TextCoordinate| {
        vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y) + vec2(0.5 * delta_x, 0.5 * delta_y)
    };
    let elements = drawables
        .into_iter()
        .map(|op| stroke_opset(op, color))
        .collect::<Vec<_>>();
    let background = background.parse::<LinColor>().ok();
    let scene = Scene::group(elements);
    let mut image = scene.render(
        &ActiveEdgeRasterizer::default(),
        Transform::identity(),
        Some(BBox::new((0.0, 0.0), (job.width as f64, job.height as f64))),
        background,
    );
    let shape = image.shape();
    let mut im_mut = image.as_mut();
    let data_mut = im_mut.data_mut();
    let text_size = delta_x.min(delta_y) * 1.6;
    let ascent = font.as_scaled(text_size).ascent();
    for (coord, word) in labels.iter() {
        let center = pos_map(coord);
        let glyph: Glyph = font.glyph_id(word).with_scale_and_position(
            text_size,
            point(center.x - delta_x / 2.0, center.y - delta_y / 2.0 + ascent),
        );
        if let Some(q) = font.outline_glyph(glyph) {
            let bound = q.px_bounds();
            q.draw(|x, y, c| {
                let x = bound.min.x + x as f32;
                let y = bound.min.y + y as f32;
                let ndx = shape.offset(y as usize, x as usize);
                data_mut[ndx] = data_mut[ndx].lerp(color, c);
            })
        }
    }
    Ok(image)
}

#[cfg(test)]
mod tests {
    use crate::TextBuffer;

    use super::*;

    #[test]
    fn test_startup_screen() {
        let tb = TextBuffer::with_text(include_str!("startup_screen.txt"));
        let job = RenderJob::rough(tb);
        let path = std::path::Path::new(r"image.png");
        let file = std::fs::File::create(path).unwrap();
        let w = std::io::BufWriter::new(file);
        let img = render(&job, "#FFFFFF", "#000000").unwrap();
        img.write_png(w).unwrap();
    }
}
