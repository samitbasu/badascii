use roughr::core::{Drawable, OpSetType, OpType};

use crate::{
    render::{RenderJob, vec2},
    tc::TextCoordinate,
};

pub fn stroke_opset(ops: Drawable<f32>, mut painter: svg::Document, color: &str) -> svg::Document {
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
            .set("stroke", color)
            .set("stroke-width", 1)
            .set("d", data);
        painter = painter.add(path);
    }
    painter
}

pub fn render(job: &RenderJob, color: &str) -> String {
    let mut context = svg::Document::new().set("viewBox", (0.0, 0.0, job.width, job.height));
    let delta_x = job.width / job.num_cols as f32;
    let delta_y = job.height / job.num_rows as f32;
    let (labels, drawables) = job.invoke();
    let pos_map = |pos: TextCoordinate| {
        vec2(pos.x as f32 * delta_x, pos.y as f32 * delta_y) + vec2(0.5 * delta_x, 0.5 * delta_y)
    };
    for op in drawables {
        context = stroke_opset(op, context, color);
    }
    let text_size = delta_x.min(delta_y) * 1.6;
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
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;

    use crate::text_buffer::TextBuffer;

    use super::*;

    const INITIAL_TEXT: &str = "
     +---------------------+
     |                     |
+--->| data           data |o--+
|    |                     |   |
|   o| full           next |>  |
v    |                     |   |
    o| overflow  underflow |o--+
     |                     |
     +---------------------+
";

    #[test]
    fn test_svg_export() {
        let mut tb = TextBuffer::new(30, 60);
        tb.paste(INITIAL_TEXT, TextCoordinate { x: 5, y: 5 });
        let svg = crate::svg::render(
            &RenderJob {
                width: 600.0,
                height: 450.0,
                num_cols: 60,
                num_rows: 30,
                text: tb,
                options: roughr::core::Options::default(),
                x0: 0.0,
                y0: 0.0,
            },
            "white",
        );
        let expect = expect_file!["todo.svg"];
        expect.assert_eq(&svg);
    }

    #[test]
    fn test_roughr_randomness() {
        const TEST_TEXT: &str = "
                                                                             
     +---------------------+                                               
     |                     |                                               
+--->| data           data |o--+                                           
|    |                     |   |                                           
|   o| full           next |>  |                                           
v    |                     |   |                                           
    o| overflow  underflow |o--+                                           
     |                     |                                               
     +---------------------+                                               
                                                                          
                                                                          
                                                                          
                                                                          
                                              +---------------------+     
                                              |                     |     
                                         +--->| data           data |o--+ 
                                         |    |                     |   | 
                                         |   o| full           next |>  | 
                                         v    |                     |   | 
                                             o| overflow  underflow |o--+ 
     +---------------------+                  |                     |     
     |                     |                  +---------------------+     
+--->| data           data |o--+                                          
|    |                     |   |                                          
|   o| full           next |>  |                                          
v    |                     |   |                                          
    o| overflow  underflow |o--+                                          
     |                     |                                              
     +---------------------+                                              
                                                                                  
     ";
        let mut tb = TextBuffer::new(40, 100);
        tb.paste(TEST_TEXT, TextCoordinate { x: 5, y: 5 });
        let svg = crate::svg::render(
            &RenderJob {
                width: 1000.0,
                height: 40.0 * 15.0,
                num_cols: 100,
                num_rows: 40,
                text: tb,
                options: roughr::core::Options::default(),
                x0: 0.0,
                y0: 0.0,
            },
            "white",
        );
        expect_file!["rough.svg"].assert_eq(&svg);
    }
}
