#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::{egui, glow::COLOR_RENDERABLE};
use egui::{
    Align2, Color32, CursorIcon, Event, EventFilter, FontId, Key, Modifiers, Painter, Pos2, Rect,
    Response, Sense, Stroke, UiBuilder, epaint::PathStroke, pos2, text, vec2,
};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

#[derive(Copy, Clone, Debug)]
struct TextCoordinate {
    x: u32,
    y: u32,
}

impl TextCoordinate {
    fn right(self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
        }
    }
    fn left(self) -> Self {
        Self {
            x: self.x.saturating_sub(1),
            y: self.y,
        }
    }
    fn up(self) -> Self {
        Self {
            x: self.x,
            y: self.y.saturating_sub(1),
        }
    }
    fn down(self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }

    fn shifted(self, origin: TextCoordinate, move_pos: TextCoordinate) -> TextCoordinate {
        let delta_x = move_pos.x as i32 - origin.x as i32;
        let delta_y = move_pos.y as i32 - origin.y as i32;
        TextCoordinate {
            x: self.x.saturating_add_signed(delta_x),
            y: self.y.saturating_add_signed(delta_y),
        }
    }

    fn perp_align(self, last_pos: TextCoordinate) -> TextCoordinate {
        let delta_x = self.x as i32 - last_pos.x as i32;
        let delta_y = self.y as i32 - last_pos.y as i32;
        if delta_x.abs() < delta_y.abs() {
            Self {
                x: last_pos.x,
                y: self.y,
            }
        } else {
            Self {
                x: self.x,
                y: last_pos.y,
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct TextState {
    origin: TextCoordinate,
    cursor: TextCoordinate,
}

#[derive(Copy, Clone, Debug)]
struct MoveState {
    selection: Rectangle,
    origin: TextCoordinate,
    move_pos: TextCoordinate,
}

#[derive(Clone, Debug, Default)]
struct LineState {
    anchors: Vec<TextCoordinate>,
    cursor: Option<TextCoordinate>,
}

#[derive(Clone, Debug)]
enum SelectedTool {
    Selection(Option<TextCoordinate>),
    Rect(Option<TextCoordinate>),
    Text(Option<TextState>),
    Selected(Rectangle),
    Moving(MoveState),
    Polyline(LineState),
}

#[derive(Copy, Clone, Debug)]
struct Rectangle {
    corner_1: TextCoordinate,
    corner_2: TextCoordinate,
}

impl Rectangle {
    fn new(corner_1: TextCoordinate, corner_2: TextCoordinate) -> Rectangle {
        Rectangle { corner_1, corner_2 }
    }
    fn contains(&self, coord: &TextCoordinate) -> bool {
        let min_x = self.corner_1.x.min(self.corner_2.x);
        let max_x = self.corner_1.x.max(self.corner_2.x);
        let min_y = self.corner_1.y.min(self.corner_2.y);
        let max_y = self.corner_1.y.max(self.corner_2.y);
        (min_x..=max_x).contains(&coord.x) && (min_y..=max_y).contains(&coord.y)
    }

    fn shifted(self, origin: TextCoordinate, move_pos: TextCoordinate) -> Self {
        Self {
            corner_1: self.corner_1.shifted(origin, move_pos),
            corner_2: self.corner_2.shifted(origin, move_pos),
        }
    }

    fn iter(&self) -> impl Iterator<Item = TextCoordinate> {
        let min_x = self.corner_1.x.min(self.corner_2.x);
        let max_x = self.corner_1.x.max(self.corner_2.x);
        let min_y = self.corner_1.y.min(self.corner_2.y);
        let max_y = self.corner_1.y.max(self.corner_2.y);
        (min_y..=max_y).flat_map(move |y| (min_x..=max_x).map(move |x| TextCoordinate { x, y }))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Action {
    Char(char),
    Backspace,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    Escape,
    Enter,
}

fn map_key(key: &Key, modifiers: &Modifiers) -> Option<Action> {
    let ret = match key {
        Key::Space => Some(' '),
        Key::Questionmark => Some('?'),
        Key::Slash => Some('/'),
        Key::Colon => Some(':'),
        Key::Backtick => Some('`'),
        Key::CloseBracket => Some('['),
        Key::Exclamationmark => Some('!'),
        Key::Plus => Some('+'),
        Key::Minus => Some('-'),
        Key::Pipe => Some('|'),
        Key::Backspace => return Some(Action::Backspace),
        Key::ArrowUp => return Some(Action::UpArrow),
        Key::ArrowDown => return Some(Action::DownArrow),
        Key::ArrowLeft => return Some(Action::LeftArrow),
        Key::ArrowRight => return Some(Action::RightArrow),
        Key::Escape => return Some(Action::Escape),
        Key::Enter => return Some(Action::Enter),
        _ => key.name().chars().next(),
    };
    let ret = if !modifiers.shift {
        ret.map(|c| c.to_ascii_lowercase())
    } else {
        ret
    };
    ret.map(Action::Char)
}

#[derive(Clone, Debug)]
struct TextBuffer {
    buffer: Box<[Option<char>]>,
    num_rows: u32,
    num_cols: u32,
}

impl TextBuffer {
    pub fn new(rows: u32, cols: u32) -> Self {
        Self {
            buffer: vec![None; (cols * rows) as usize].into_boxed_slice(),
            num_rows: rows,
            num_cols: cols,
        }
    }
    pub fn set_text(&mut self, pos: &TextCoordinate, ch: Option<char>) {
        if (0..self.num_cols).contains(&pos.x) && (0..self.num_rows).contains(&pos.y) {
            self.buffer[(pos.x + pos.y * self.num_cols) as usize] = ch;
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (TextCoordinate, char)> {
        self.buffer.iter().enumerate().filter_map(|(ndx, c)| {
            if let Some(c) = c {
                let row = ndx as u32 / self.num_cols;
                let col = ndx as u32 % self.num_cols;
                Some((TextCoordinate { x: col, y: row }, *c))
            } else {
                None
            }
        })
    }

    fn clear_rectangle(&mut self, selection: Rectangle) {
        for pos in selection.iter() {
            self.set_text(&pos, None);
        }
    }

    fn get(&self, pos: TextCoordinate) -> Option<char> {
        if (0..self.num_cols).contains(&pos.x) && (0..self.num_rows).contains(&pos.y) {
            self.buffer[(pos.x + pos.y * self.num_cols) as usize]
        } else {
            None
        }
    }

    fn clear_all(&mut self) {
        self.buffer.fill(None)
    }
}

struct MyApp {
    num_rows: u32,
    num_cols: u32,
    tool: SelectedTool,
    rects: Vec<Rectangle>,
    lines: Vec<Vec<TextCoordinate>>,
    selected_text: TextBuffer,
    text: TextBuffer,
}

impl Default for MyApp {
    fn default() -> Self {
        let num_rows = 40;
        let num_cols = 100;
        Self {
            num_rows,
            num_cols,
            tool: SelectedTool::Polyline(LineState::default()),
            lines: vec![],
            rects: vec![],
            selected_text: TextBuffer::new(num_rows, num_cols),
            text: TextBuffer::new(num_rows, num_cols),
        }
    }
}

impl MyApp {
    fn map_pos_to_coords(&self, canvas: &Rect, pos: Pos2) -> Option<TextCoordinate> {
        let top_left = canvas.left_top();
        let delta = pos - top_left;
        let delta_x = canvas.width() / self.num_cols as f32;
        let delta_y = canvas.height() / self.num_rows as f32;
        let col = (delta.x / delta_x).floor() as i32;
        let row = (delta.y / delta_y).floor() as i32;
        if (0..(self.num_cols as i32)).contains(&col) && (0..(self.num_rows as i32)).contains(&row)
        {
            Some(TextCoordinate {
                x: col as u32,
                y: row as u32,
            })
        } else {
            None
        }
    }
    fn map_text_coordinate_to_cell_center(&self, canvas: &Rect, coord: &TextCoordinate) -> Pos2 {
        let left_top = canvas.left_top();
        let delta_x = canvas.width() / self.num_cols as f32;
        let delta_y = canvas.height() / self.num_rows as f32;
        let left_top_corner = left_top + vec2(delta_x * coord.x as f32, delta_y * coord.y as f32);
        left_top_corner + vec2(delta_x / 2.0, delta_y / 2.0)
    }
    fn map_rectangle_to_rect(&self, canvas: &Rect, rect: &Rectangle) -> Rect {
        let corner_1 = self.map_text_coordinate_to_cell_center(canvas, &rect.corner_1);
        let corner_2 = self.map_text_coordinate_to_cell_center(canvas, &rect.corner_2);
        Rect::from_two_pos(corner_1, corner_2)
    }
    fn set_text(&mut self, ch: char, position: &TextCoordinate) {
        self.text.set_text(position, Some(ch));
    }
    fn clear_text(&mut self, position: &TextCoordinate) {
        self.text.set_text(position, None);
    }
    fn on_drag_start(&mut self, tc: TextCoordinate) {
        match &self.tool {
            SelectedTool::Rect(None) => self.tool = SelectedTool::Rect(Some(tc)),
            SelectedTool::Selection(None) => self.tool = SelectedTool::Selection(Some(tc)),
            SelectedTool::Selected(rect) => {
                self.tool = SelectedTool::Moving(MoveState {
                    selection: *rect,
                    origin: tc,
                    move_pos: tc,
                })
            }
            _ => (),
        }
    }
    fn on_drag(&mut self, corner2: TextCoordinate, canvas: &Rect, painter: &Painter) {
        let delta_x = canvas.width() / self.num_cols as f32;
        let delta_y = canvas.height() / self.num_rows as f32;
        match &self.tool {
            SelectedTool::Rect(Some(corner1)) => {
                let text_box = Rectangle::new(*corner1, corner2);
                let rect = self.map_rectangle_to_rect(canvas, &text_box);
                painter.rect_stroke(
                    rect,
                    1.0,
                    Stroke::new(1.0, Color32::WHITE),
                    egui::StrokeKind::Middle,
                );
            }
            SelectedTool::Selection(Some(corner1)) => {
                let selection_box = Rectangle::new(*corner1, corner2);
                let rect = self.map_rectangle_to_rect(canvas, &selection_box);
                let rect = rect.expand2(vec2(delta_x / 2.0, delta_y / 2.0));
                painter.rect_stroke(
                    rect,
                    1.0,
                    (1.0, Color32::LIGHT_GREEN),
                    egui::StrokeKind::Middle,
                );
            }
            SelectedTool::Moving(MoveState {
                selection,
                origin,
                move_pos: _,
            }) => {
                self.tool = SelectedTool::Moving(MoveState {
                    selection: *selection,
                    origin: *origin,
                    move_pos: corner2,
                });
            }
            _ => {}
        }
    }
    fn on_drag_stop(&mut self, corner2: TextCoordinate) {
        match &self.tool {
            SelectedTool::Rect(Some(corner1)) => {
                let text_box = Rectangle::new(*corner1, corner2);
                self.rects.push(text_box);
                self.tool = SelectedTool::Rect(None);
            }
            SelectedTool::Selection(Some(corner1)) => {
                let selection = Rectangle::new(*corner1, corner2);
                self.selected_text = self.text.clone();
                self.text.clear_rectangle(selection);
                self.tool = SelectedTool::Selected(selection);
            }
            SelectedTool::Moving(MoveState {
                selection,
                origin,
                move_pos,
            }) => {
                for pos in selection.iter() {
                    let selection = self.selected_text.get(pos);
                    let new_pos = pos.shifted(*origin, *move_pos);
                    self.text.set_text(&new_pos, selection);
                }
                self.selected_text.clear_all();
                self.tool = SelectedTool::Selection(None)
            }
            _ => {}
        }
    }
    fn on_click(&mut self, pos: TextCoordinate) {
        match &self.tool {
            SelectedTool::Text(_) => {
                self.tool = SelectedTool::Text(Some(TextState {
                    origin: pos,
                    cursor: pos,
                }))
            }
            SelectedTool::Polyline(state) => {
                let mut state = state.clone();
                let mut txt_pos = pos;
                if let Some(last_pos) = state.anchors.last() {
                    txt_pos = txt_pos.perp_align(*last_pos);
                }
                state.anchors.push(txt_pos);
                self.tool = SelectedTool::Polyline(state);
            }
            _ => {}
        }
    }
    fn on_action_with_text(&mut self, text_state: TextState, action: Action) {
        let TextState { cursor, origin } = text_state;
        match action {
            Action::Backspace => {
                self.clear_text(&cursor);
                self.tool = SelectedTool::Text(Some(TextState {
                    origin,
                    cursor: cursor.left(),
                }));
            }
            Action::Char(ch) => {
                self.set_text(ch, &cursor);
                self.tool = SelectedTool::Text(Some(TextState {
                    origin,
                    cursor: cursor.right(),
                }));
            }
            Action::RightArrow => {
                self.tool = SelectedTool::Text(Some(TextState {
                    origin,
                    cursor: cursor.right(),
                }));
            }
            Action::LeftArrow => {
                self.tool = SelectedTool::Text(Some(TextState {
                    origin,
                    cursor: cursor.left(),
                }));
            }
            Action::UpArrow => {
                self.tool = SelectedTool::Text(Some(TextState {
                    origin,
                    cursor: cursor.up(),
                }));
            }
            Action::DownArrow => {
                self.tool = SelectedTool::Text(Some(TextState {
                    origin,
                    cursor: cursor.down(),
                }));
            }
            Action::Escape => {
                self.tool = SelectedTool::Selection(None);
            }
            Action::Enter => {
                let origin = origin.down();
                self.tool = SelectedTool::Text(Some(TextState {
                    origin,
                    cursor: origin,
                }));
            }
        }
    }
    fn on_action(&mut self, action: Action) {
        match &self.tool {
            SelectedTool::Text(Some(text_state)) => {
                self.on_action_with_text(*text_state, action);
            }
            SelectedTool::Selection(None) => match action {
                Action::Char('r') => self.tool = SelectedTool::Rect(None),
                Action::Char('t') => self.tool = SelectedTool::Text(None),
                Action::Char('w') => self.tool = SelectedTool::Polyline(LineState::default()),
                _ => {}
            },
            SelectedTool::Polyline(state) => {
                if action == Action::Escape {
                    if state.anchors.is_empty() {
                        self.tool = SelectedTool::Selection(None)
                    } else {
                        self.lines.push(state.anchors.clone());
                        self.tool = SelectedTool::Polyline(LineState::default())
                    }
                }
            }
            _ if action == Action::Escape => self.tool = SelectedTool::Selection(None),
            _ => {}
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                ui.label(format!("{:?}", self.tool));
                let desired_size = ui.available_size();
                let (resp, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());
                let canvas = resp.rect;
                let delta_x = desired_size.x / self.num_cols as f32;
                let delta_y = desired_size.y / self.num_rows as f32;
                let top_left = canvas.left_top();
                for column in 0..=self.num_cols {
                    let col_x = column as f32 * delta_x;
                    let p0 = top_left + vec2(col_x, 0.0);
                    let p1 = top_left + vec2(col_x, canvas.height());
                    painter.line(
                        vec![p0, p1],
                        PathStroke::new(1.0, Color32::from_gray(65).linear_multiply(0.5)),
                    );
                }
                for row in 0..=self.num_rows {
                    let row_y = row as f32 * delta_y;
                    let p0 = top_left + vec2(0.0, row_y);
                    let p1 = top_left + vec2(canvas.width(), row_y);
                    painter.line(
                        vec![p0, p1],
                        PathStroke::new(1.0, Color32::from_gray(65).linear_multiply(0.5)),
                    );
                }
                for rect in &self.rects {
                    let rect = self.map_rectangle_to_rect(&canvas, rect);
                    painter.rect_stroke(
                        rect,
                        1.0,
                        (1.0, Color32::LIGHT_BLUE),
                        egui::StrokeKind::Middle,
                    );
                }
                for line in &self.lines {
                    let points = line
                        .iter()
                        .map(|t| self.map_text_coordinate_to_cell_center(&canvas, t))
                        .collect::<Vec<_>>();
                    painter.line(points, (1.0, Color32::LIGHT_BLUE));
                }
                for (coord, ch) in self.text.iter() {
                    let center = self.map_text_coordinate_to_cell_center(&canvas, &coord);
                    let monospace = FontId::monospace(10.0);
                    painter.text(
                        center,
                        Align2::CENTER_CENTER,
                        ch,
                        monospace,
                        Color32::WHITE.linear_multiply(0.7),
                    );
                }
                if let Some(pos) = resp.hover_pos() {
                    if let Some(text_coordinate) = self.map_pos_to_coords(&canvas, pos) {
                        let col = text_coordinate.x as f32;
                        let row = text_coordinate.y as f32;
                        let top_left_corner = top_left + vec2(col * delta_x, row * delta_y);
                        let bottom_right_corner = top_left_corner + vec2(delta_x, delta_y);
                        painter.rect_filled(
                            Rect::from_two_pos(top_left_corner, bottom_right_corner),
                            1.0,
                            Color32::LIGHT_BLUE.linear_multiply(0.3),
                        );
                        if let SelectedTool::Polyline(state) = &mut self.tool {
                            state.cursor = Some(text_coordinate);
                        }
                    }
                }
                if let Some(pos) = resp.interact_pointer_pos() {
                    if let Some(text_coordinate) = self.map_pos_to_coords(&canvas, pos) {
                        if resp.drag_started() {
                            self.on_drag_start(text_coordinate);
                        } else if resp.dragged() {
                            self.on_drag(text_coordinate, &canvas, &painter);
                        } else if resp.drag_stopped() {
                            self.on_drag_stop(text_coordinate);
                        } else if resp.clicked() {
                            self.on_click(text_coordinate);
                        }
                    }
                }
                if let Some(action) = ui.input(|i| {
                    i.events.iter().find_map(|x| match x {
                        Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } => map_key(key, modifiers),
                        _ => None,
                    })
                }) {
                    self.on_action(action);
                }

                if let SelectedTool::Text(Some(TextState { origin, cursor })) = self.tool {
                    let center = self.map_text_coordinate_to_cell_center(&canvas, &cursor);
                    let rect = Rect::from_center_size(center, vec2(delta_x, delta_y));
                    painter.rect_stroke(
                        rect,
                        0.5,
                        (1.0, Color32::LIGHT_YELLOW),
                        egui::StrokeKind::Middle,
                    );
                }
                if let SelectedTool::Selected(selection_box) = self.tool {
                    let rect = self.map_rectangle_to_rect(&canvas, &selection_box);
                    let rect = rect.expand2(vec2(delta_x / 2.0, delta_y / 2.0));
                    painter.rect_stroke(
                        rect,
                        1.0,
                        (1.0, Color32::LIGHT_GREEN),
                        egui::StrokeKind::Middle,
                    );
                    for (coord, ch) in self.selected_text.iter() {
                        if selection_box.contains(&coord) {
                            let center = self.map_text_coordinate_to_cell_center(&canvas, &coord);
                            let monospace = FontId::monospace(10.0);
                            painter.text(
                                center,
                                Align2::CENTER_CENTER,
                                ch,
                                monospace,
                                Color32::WHITE.linear_multiply(0.5),
                            );
                        }
                    }
                }
                if let SelectedTool::Polyline(state) = &self.tool {
                    if let Some(last_pos) = state.anchors.last() {
                        if let Some(cursor) = state.cursor {
                            let cursor = cursor.perp_align(*last_pos);
                            let cursor = self.map_text_coordinate_to_cell_center(&canvas, &cursor);
                            let points = state
                                .anchors
                                .iter()
                                .map(|x| self.map_text_coordinate_to_cell_center(&canvas, x))
                                .chain(std::iter::once(cursor))
                                .collect();
                            painter.line(points, (1.0, Color32::DARK_RED));
                        }
                    }
                }
                if let SelectedTool::Moving(MoveState {
                    selection,
                    origin,
                    move_pos,
                }) = self.tool
                {
                    let bbox_shifted = selection.shifted(origin, move_pos);
                    painter.rect_stroke(
                        self.map_rectangle_to_rect(&canvas, &bbox_shifted)
                            .expand2(vec2(delta_x / 2.0, delta_y / 2.0)),
                        1.0,
                        (1.0, Color32::LIGHT_GREEN),
                        egui::StrokeKind::Middle,
                    );
                    for (coord, ch) in self.selected_text.iter() {
                        let coord = coord.shifted(origin, move_pos);
                        if bbox_shifted.contains(&coord) {
                            let center = self.map_text_coordinate_to_cell_center(&canvas, &coord);
                            let monospace = FontId::monospace(10.0);
                            painter.text(
                                center,
                                Align2::CENTER_CENTER,
                                ch,
                                monospace,
                                Color32::WHITE,
                            );
                        }
                    }
                }
            });
            match &self.tool {
                SelectedTool::Rect(_) | SelectedTool::Polyline(_) => {
                    ctx.set_cursor_icon(CursorIcon::Crosshair);
                }
                SelectedTool::Text(_) => {
                    ctx.set_cursor_icon(CursorIcon::Text);
                }
                SelectedTool::Selected(..) => {
                    ctx.set_cursor_icon(CursorIcon::Grab);
                }
                SelectedTool::Moving(..) => {
                    ctx.set_cursor_icon(CursorIcon::Grabbing);
                }
                _ => {
                    ctx.set_cursor_icon(CursorIcon::Default);
                }
            }
        });
    }
}
