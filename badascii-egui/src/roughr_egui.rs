use egui::{
    Color32, Painter, Pos2, Shape,
    epaint::{CubicBezierShape, PathStroke},
    pos2,
};
use roughr::{
    PathSegment,
    core::{Drawable, OpSetType, OpType},
};

pub fn move_to(p: Pos2) -> PathSegment {
    PathSegment::MoveTo {
        abs: true,
        x: p.x as f64,
        y: p.y as f64,
    }
}

pub fn line_to(p: Pos2) -> PathSegment {
    PathSegment::LineTo {
        abs: true,
        x: p.x as f64,
        y: p.y as f64,
    }
}

pub fn close_path() -> PathSegment {
    PathSegment::ClosePath { abs: true }
}

pub fn stroke_opset(ops: Drawable<f32>, painter: &Painter) {
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
                    painter.line_segment([pos, new_pos], (1.0, Color32::LIGHT_GREEN));
                    pos = new_pos;
                }
                OpType::BCurveTo => {
                    let cp1 = pos2(op.data[0], op.data[1]);
                    let cp2 = pos2(op.data[2], op.data[3]);
                    let end = pos2(op.data[4], op.data[5]);
                    painter.add(Shape::CubicBezier(CubicBezierShape {
                        points: [pos, cp1, cp2, end],
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: PathStroke::new(1.0, Color32::LIGHT_GREEN),
                    }));
                    pos = end;
                }
            }
        }
    }
}
