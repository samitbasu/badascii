#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

// Needs:
// SVG export - maybe with roughr

use std::collections::VecDeque;

use badascii::{
    rect::Rectangle, render::RenderJob, tc::TextCoordinate, text_buffer::Size,
    text_buffer::TextBuffer,
};

use badascii_gui::{action::Action, roughr_egui::stroke_opset};

const TEXT_SCALE_FACTOR: f32 = 1.5;
use eframe::egui;
use egui::{
    Align2, Button, Checkbox, Color32, CursorIcon, DragValue, Event, FontId, Key, Modifiers,
    Painter, Pos2, Rect, Response, Scene, Sense, Ui, Vec2, epaint::PathStroke,
    global_theme_preference_switch, util::hash, vec2,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "BadAscii",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
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

#[derive(Clone, Debug)]
enum Tool {
    Selection(Option<TextCoordinate>),
    Text(Option<TextState>),
    Selected(Rectangle),
    MovingText(MoveState),
}

fn map_key(key: &Key, modifiers: &Modifiers) -> Option<Action> {
    match key {
        Key::Backspace => Some(Action::Backspace),
        Key::ArrowUp if modifiers.shift_only() => Some(Action::UpControlArrow),
        Key::ArrowDown if modifiers.shift_only() => Some(Action::DownControlArrow),
        Key::ArrowLeft if modifiers.shift_only() => Some(Action::LeftControlArrow),
        Key::ArrowRight if modifiers.shift_only() => Some(Action::RightControlArrow),
        Key::ArrowUp if !modifiers.any() => Some(Action::UpArrow),
        Key::ArrowDown if !modifiers.any() => Some(Action::DownArrow),
        Key::ArrowLeft if !modifiers.any() => Some(Action::LeftArrow),
        Key::ArrowRight if !modifiers.any() => Some(Action::RightArrow),
        Key::Escape => Some(Action::Escape),
        Key::Enter => Some(Action::Enter),
        Key::Copy => Some(Action::Copy),
        _ => None,
    }
}

#[derive(Clone)]
struct Snapshot {
    text: TextBuffer,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Tab {
    Ascii,
    Preview,
}

struct MyApp {
    num_rows: u32,
    num_cols: u32,
    tool: Tool,
    snapshots: VecDeque<Snapshot>,
    futures: Vec<Snapshot>,
    selected_text: TextBuffer,
    text: TextBuffer,
    copy_buffer: Option<String>,
    hover_pos: Option<TextCoordinate>,
    resize: Option<Size>,
    prev_action: Option<Action>,
    dock_state: DockState<Tab>,
    scene_rect: Rect,
    drag_delta: Option<Vec2>,
    rough_mode: bool,
    reset_zoom: bool,
}

const INITIAL_TEXT: &str = include_str!("startup_screen.txt");

impl Default for MyApp {
    fn default() -> Self {
        let num_rows = 40;
        let num_cols = 100;
        let mut text = TextBuffer::new(num_rows, num_cols);
        text.paste(INITIAL_TEXT, TextCoordinate { x: 0, y: 0 });
        let mut state = DockState::new(vec![Tab::Ascii]);
        let surface = state.main_surface_mut();
        surface.split_right(NodeIndex::root(), 0.7, vec![Tab::Preview]);
        Self {
            snapshots: VecDeque::with_capacity(100),
            futures: Vec::new(),
            num_rows,
            num_cols,
            tool: Tool::Selection(None),
            selected_text: TextBuffer::new(num_rows, num_cols),
            text,
            copy_buffer: None,
            hover_pos: None,
            resize: None,
            prev_action: None,
            dock_state: state,
            scene_rect: Rect::NAN,
            drag_delta: None,
            rough_mode: true,
            reset_zoom: false,
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
    fn snapshot(&mut self) {
        while self.snapshots.len() >= 100 {
            self.snapshots.pop_front();
        }
        let mut text = self.text.clone();
        for (pos, c) in self.selected_text.iter() {
            text.set_text(&pos, Some(c))
        }
        let text_hash = hash(&text);
        let last_hash = self.snapshots.back().map(|t| hash(&t.text)).unwrap_or(!0);
        if text_hash != last_hash {
            self.snapshots.push_back(Snapshot { text });
        }
    }
    fn set_text(&mut self, ch: char, position: &TextCoordinate) {
        self.text.set_text(position, Some(ch));
    }
    fn clear_text(&mut self, position: &TextCoordinate) {
        self.text.set_text(position, None);
    }
    fn on_drag_start(&mut self, tc: TextCoordinate, resp: &Response) {
        match &self.tool {
            Tool::Selection(None) => {
                if !resp.dragged_by(egui::PointerButton::Secondary) {
                    self.tool = Tool::Selection(Some(tc));
                }
            }
            Tool::Selected(rect) => {
                self.tool = Tool::MovingText(MoveState {
                    selection: *rect,
                    origin: tc,
                    move_pos: tc,
                })
            }
            Tool::Text(_) => self.tool = Tool::Selection(Some(tc)),
            _ => (),
        }
    }
    fn on_drag(&mut self, corner2: TextCoordinate, canvas: &Rect, painter: &Painter) {
        let delta_x = canvas.width() / self.num_cols as f32;
        let delta_y = canvas.height() / self.num_rows as f32;
        match &self.tool {
            Tool::Selection(Some(corner1)) => {
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
            Tool::MovingText(MoveState {
                selection,
                origin,
                move_pos: _,
            }) => {
                self.tool = Tool::MovingText(MoveState {
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
            Tool::Selection(Some(corner1)) => {
                let selection = Rectangle::new(*corner1, corner2);
                if selection
                    .iter_interior()
                    .any(|pos| self.text.get(pos).is_some())
                {
                    self.snapshot();
                    self.selected_text = self.text.clone();
                    self.text.clear_rectangle(selection);
                    self.tool = Tool::Selected(selection);
                } else {
                    self.tool = Tool::Selection(None);
                }
            }
            Tool::MovingText(MoveState {
                selection,
                origin,
                move_pos,
            }) => {
                let mut swap_buf = TextBuffer::new(self.num_rows, self.num_cols);
                for pos in selection.iter_interior() {
                    let selection = self.selected_text.get(pos);
                    let new_pos = pos.shifted(*origin, *move_pos);
                    swap_buf.merge_text(&new_pos, selection);
                }
                let selection_shifted = selection.shifted(*origin, *move_pos);
                self.snapshot();
                self.selected_text = swap_buf;
                self.tool = Tool::Selected(selection_shifted);
            }
            _ => {}
        }
    }
    fn on_click(&mut self, pos: TextCoordinate) {
        match &self.tool {
            Tool::Text(_) => {
                self.snapshot();
                self.tool = Tool::Text(Some(TextState {
                    origin: pos,
                    cursor: pos,
                }))
            }
            Tool::Selected(selection_box) => {
                let selection_box = *selection_box;
                self.snapshot();
                for pos in selection_box.iter_interior() {
                    let selection = self.selected_text.get(pos);
                    self.text.merge_text(&pos, selection);
                }
                self.selected_text.clear_all();
                self.tool = Tool::Selection(None);
            }
            Tool::Selection(None) => {
                self.tool = Tool::Text(Some(TextState {
                    origin: pos,
                    cursor: pos,
                }));
            }
            _ => {}
        }
    }
    fn on_action_with_text(&mut self, text_state: TextState, action: Action) {
        let TextState { cursor, origin } = text_state;
        match action.clone() {
            Action::Paste(txt) => {
                self.text.paste(&txt, cursor);
            }
            Action::Backspace => {
                self.clear_text(&cursor);
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.left(),
                }));
            }
            Action::Char(ch) => {
                self.set_text(ch, &cursor);
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.right(),
                }));
            }
            Action::RightControlArrow => {
                let char = if (self.prev_action != Some(Action::RightControlArrow))
                    && (self.prev_action != Some(Action::LeftControlArrow))
                {
                    '+'
                } else {
                    '-'
                };
                self.set_text(char, &cursor);
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.right(),
                }));
            }
            Action::RightArrow => {
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.right(),
                }));
            }
            Action::LeftControlArrow => {
                let char = if (self.prev_action != Some(Action::RightControlArrow))
                    && (self.prev_action != Some(Action::LeftControlArrow))
                {
                    '+'
                } else {
                    '-'
                };
                self.set_text(char, &cursor);
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.left(),
                }));
            }
            Action::LeftArrow => {
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.left(),
                }));
            }
            Action::UpControlArrow => {
                let char = if (self.prev_action != Some(Action::UpControlArrow))
                    && (self.prev_action != Some(Action::DownControlArrow))
                {
                    '+'
                } else {
                    '|'
                };
                self.set_text(char, &cursor);
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.up(),
                }));
            }
            Action::UpArrow => {
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.up(),
                }));
            }
            Action::DownControlArrow => {
                let char = if (self.prev_action != Some(Action::DownControlArrow))
                    && (self.prev_action != Some(Action::UpControlArrow))
                {
                    '+'
                } else {
                    '|'
                };
                self.set_text(char, &cursor);
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.down(),
                }));
            }
            Action::DownArrow => {
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: cursor.down(),
                }));
            }
            Action::Escape => {
                self.tool = Tool::Selection(None);
            }
            Action::Enter => {
                let origin = origin.down();
                self.tool = Tool::Text(Some(TextState {
                    origin,
                    cursor: origin,
                }));
            }
            Action::Copy => {
                self.copy_buffer = Some(self.text.render());
            }
        }
        self.prev_action = Some(action);
    }
    fn on_action(&mut self, action: Action) {
        match &self.tool {
            Tool::Text(Some(text_state)) => {
                self.on_action_with_text(*text_state, action);
            }
            Tool::Selection(None) => match action {
                Action::Char('t') => self.tool = Tool::Text(None),
                Action::Copy => {
                    self.copy_buffer = Some(self.text.render());
                }
                Action::Paste(txt) => {
                    self.snapshot();
                    let hover_pos = self.hover_pos.unwrap_or_default();
                    let rect = self.selected_text.paste(&txt, hover_pos);
                    self.tool = Tool::Selected(rect);
                }
                _ => {}
            },
            Tool::Selected(rect) if action == Action::Copy => {
                let selection = self.selected_text.window(rect);
                self.copy_buffer = Some(selection.render());
            }
            Tool::Selected(rect) if action == Action::Escape => {
                for pos in rect.iter_interior() {
                    let selection = self.selected_text.get(pos);
                    self.text.merge_text(&pos, selection);
                }
                self.selected_text.clear_all();
                self.tool = Tool::Selection(None);
            }
            Tool::Selected(_) if action == Action::Backspace => {
                self.selected_text.clear_all();
                self.tool = Tool::Selection(None);
            }
            _ if action == Action::Escape => self.tool = Tool::Selection(None),
            _ => {}
        }
    }
    fn on_hover(&mut self, tc: Option<TextCoordinate>) {
        self.hover_pos = tc;
    }
    fn undo(&mut self) {
        if let Some(buf) = self.snapshots.pop_back() {
            self.futures.push(buf.clone());
            self.text = buf.text;
            self.selected_text.clear_all();
            self.tool = Tool::Selection(None);
        }
    }
    fn redo(&mut self) {
        if let Some(buf) = self.futures.pop() {
            self.text = buf.text;
            self.selected_text.clear_all();
            self.tool = Tool::Selection(None);
            self.snapshot();
        }
    }
    fn ascii_control_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            global_theme_preference_switch(ui);
            if ui.button("⚙").clicked() {
                self.resize = Some(Size {
                    num_cols: self.num_cols,
                    num_rows: self.num_rows,
                });
            }
            if ui
                .add_enabled(!self.snapshots.is_empty(), Button::new("Undo"))
                .clicked()
            {
                self.undo();
            }
            if ui
                .add_enabled(!self.futures.is_empty(), Button::new("Redo"))
                .clicked()
            {
                self.redo();
            }
            if ui.button("📋").clicked() {
                let ascii = self.text.render();
                ui.output_mut(|o| o.commands.push(egui::OutputCommand::CopyText(ascii)))
            }
        });
    }
    fn preview_control_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            global_theme_preference_switch(ui);
            ui.add(Checkbox::new(&mut self.rough_mode, "Rough Sketch"));
            if ui.button("📋").clicked() {
                let job = RenderJob {
                    width: self.num_cols as f32 * 10.0,
                    height: self.num_rows as f32 * 15.0,
                    text: self.text.clone(),
                    options: self.roughr_options(),
                    x0: 0.0,
                    y0: 0.0,
                };
                let text_color = ui.visuals().strong_text_color().to_hex();
                let svg = badascii::svg::render(&job, &text_color);
                ui.output_mut(|o| o.commands.push(egui::OutputCommand::CopyText(svg)))
            }
        });
    }
    fn resize_panel(&mut self, ui: &mut Ui) {
        if let Some(mut resize) = self.resize.take() {
            let mut should_close = false;
            let mut should_apply = false;
            let modal = egui::containers::Modal::new("Resize".into());
            modal.show(ui.ctx(), |ui| {
                ui.label("Resize canvas");
                egui::Grid::new("resize")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Width");
                        ui.add(DragValue::new(&mut resize.num_cols));
                        ui.end_row();
                        ui.label("Height");
                        ui.add(DragValue::new(&mut resize.num_rows));
                        ui.end_row();
                    });

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                        if ui.button("Apply").clicked() {
                            should_apply = true;
                            should_close = true;
                        }
                    },
                );
            });
            if should_close {
                if should_apply {
                    self.num_cols = resize.num_cols;
                    self.num_rows = resize.num_rows;
                    self.text = self.text.resize(resize);
                }
                self.resize = None;
            } else {
                self.resize = Some(resize);
            }
        };
    }
    fn draw_grid(&mut self, canvas: &Rect, painter: &Painter, grid_color: Color32) {
        let delta_x = canvas.width() / self.num_cols as f32;
        let delta_y = canvas.height() / self.num_rows as f32;
        let top_left = canvas.left_top();
        for column in 0..=self.num_cols {
            let col_x = column as f32 * delta_x;
            let p0 = top_left + vec2(col_x, 0.0);
            let p1 = top_left + vec2(col_x, canvas.height());
            painter.line(vec![p0, p1], PathStroke::new(1.0, grid_color));
        }
        for row in 0..=self.num_rows {
            let row_y = row as f32 * delta_y;
            let p0 = top_left + vec2(0.0, row_y);
            let p1 = top_left + vec2(canvas.width(), row_y);
            painter.line(vec![p0, p1], PathStroke::new(1.0, grid_color));
        }
    }
    fn draw_text_buffer(&mut self, canvas: &Rect, painter: &Painter, text_color: Color32) {
        let delta_x = canvas.width() / self.num_cols as f32;
        let delta_y = canvas.height() / self.num_rows as f32;
        let text_size = delta_x.min(delta_y) * TEXT_SCALE_FACTOR;
        let monospace = FontId::monospace(text_size);
        for (coord, ch) in self.text.iter() {
            let center = self.map_text_coordinate_to_cell_center(canvas, &coord);
            painter.text(
                center,
                Align2::CENTER_CENTER,
                ch,
                monospace.clone(),
                text_color,
            );
        }
    }
    fn roughr_options(&self) -> roughr::core::Options {
        if self.rough_mode {
            roughr::core::Options::default()
        } else {
            roughr::core::Options {
                disable_multi_stroke: Some(true),
                max_randomness_offset: Some(0.0),
                roughness: Some(0.0),
                ..Default::default()
            }
        }
    }
    fn draw_rendered_schematic(&mut self, canvas: &Rect, painter: &Painter, color: Color32) {
        let top_left = canvas.left_top();
        let mut text = self.text.clone();
        if let Tool::Selected(_rect) = &self.tool {
            for (pos, c) in self.selected_text.iter() {
                text.set_text(&pos, Some(c))
            }
        }
        let job = RenderJob {
            width: canvas.width(),
            height: canvas.height(),
            text,
            options: self.roughr_options(),
            x0: top_left.x,
            y0: top_left.y,
        };
        let (tb, ops) = job.invoke();
        for op in ops {
            stroke_opset(op, painter, color);
        }
        let delta_x = canvas.width() / self.num_cols as f32;
        let delta_y = canvas.height() / self.num_rows as f32;
        let text_size = delta_x.min(delta_y) * TEXT_SCALE_FACTOR;
        let monospace = FontId::monospace(text_size);
        for (coord, ch) in tb.iter() {
            let center = self.map_text_coordinate_to_cell_center(canvas, &coord);
            painter.text(center, Align2::CENTER_CENTER, ch, monospace.clone(), color);
        }
    }
    fn show_hover(&mut self, canvas: &Rect, pos: Pos2, painter: &Painter) {
        let top_left = canvas.left_top();
        let delta_x = canvas.width() / self.num_cols as f32;
        let delta_y = canvas.height() / self.num_rows as f32;
        if let Some(text_coordinate) = self.map_pos_to_coords(canvas, pos) {
            let col = text_coordinate.x as f32;
            let row = text_coordinate.y as f32;
            let top_left_corner = top_left + vec2(col * delta_x, row * delta_y);
            let bottom_right_corner = top_left_corner + vec2(delta_x, delta_y);
            painter.rect_filled(
                Rect::from_two_pos(top_left_corner, bottom_right_corner),
                1.0,
                Color32::LIGHT_BLUE.linear_multiply(0.3),
            );
            self.on_hover(Some(text_coordinate));
        } else {
            self.on_hover(None);
        }
    }
    fn on_handle_interaction(
        &mut self,
        resp: &Response,
        canvas: &Rect,
        pos: Pos2,
        painter: &Painter,
    ) {
        if let Some(text_coordinate) = self.map_pos_to_coords(canvas, pos) {
            if resp.drag_started() {
                self.on_drag_start(text_coordinate, resp);
            } else if resp.dragged() {
                self.on_drag(text_coordinate, canvas, painter);
            } else if resp.drag_stopped() {
                self.on_drag_stop(text_coordinate);
            } else if resp.clicked() {
                self.on_click(text_coordinate);
            }
            if resp.dragged_by(egui::PointerButton::Secondary) {
                self.drag_delta = Some(resp.drag_delta());
            } else {
                self.drag_delta = None;
            }
        }
    }
    fn tool_specific_drawing(&self, canvas: &Rect, painter: &Painter) {
        let delta_x = canvas.width() / self.num_cols as f32;
        let delta_y = canvas.height() / self.num_rows as f32;
        let text_size = delta_x.min(delta_y) * TEXT_SCALE_FACTOR;
        let monospace = FontId::monospace(text_size);
        match self.tool {
            Tool::Text(Some(TextState { origin: _, cursor })) => {
                let center = self.map_text_coordinate_to_cell_center(canvas, &cursor);
                let rect = Rect::from_center_size(center, vec2(delta_x, delta_y));
                painter.rect_stroke(
                    rect,
                    0.5,
                    (1.0, Color32::LIGHT_YELLOW),
                    egui::StrokeKind::Middle,
                );
            }
            Tool::Selected(selection_box) => {
                for (coord, ch) in self.selected_text.iter() {
                    if selection_box.contains(&coord) {
                        let center = self.map_text_coordinate_to_cell_center(canvas, &coord);
                        painter.text(
                            center,
                            Align2::CENTER_CENTER,
                            ch,
                            monospace.clone(),
                            Color32::GREEN.linear_multiply(0.5),
                        );
                    }
                }
            }
            Tool::MovingText(MoveState {
                selection,
                origin,
                move_pos,
            }) => {
                let bbox_shifted = selection.shifted(origin, move_pos);
                for (coord, ch) in self.selected_text.iter() {
                    let coord = coord.shifted(origin, move_pos);
                    if bbox_shifted.contains(&coord) {
                        let center = self.map_text_coordinate_to_cell_center(canvas, &coord);
                        painter.text(
                            center,
                            Align2::CENTER_CENTER,
                            ch,
                            monospace.clone(),
                            Color32::GREEN,
                        );
                    }
                }
            }
            _ => {}
        }
    }
    fn process_actions(&mut self, ui: &mut Ui) {
        if let Some(action) = ui.input(|i| {
            i.events.iter().find_map(|x| match x {
                Event::Text(string) => {
                    if string.len() == 1 {
                        Some(Action::Char(string.chars().nth(0).unwrap()))
                    } else {
                        None
                    }
                }
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => map_key(key, modifiers),
                Event::Paste(string) => Some(Action::Paste(string.clone())),
                Event::Copy => Some(Action::Copy),
                _ => None,
            })
        }) {
            self.on_action(action);
        }
    }
    fn draw_ascii_widget(&mut self, ui: &mut Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let desired_size = ui.available_size();
            let (resp, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());
            let canvas = resp.rect;
            let text_color = ui.style().visuals.strong_text_color();
            let grid_color = ui.style().visuals.code_bg_color;
            self.draw_grid(&canvas, &painter, grid_color);
            self.draw_text_buffer(&canvas, &painter, text_color);
            if let Some(pos) = resp.hover_pos() {
                self.show_hover(&canvas, pos, &painter);
                match &self.tool {
                    Tool::Text(_) => {
                        ui.ctx().set_cursor_icon(CursorIcon::Text);
                    }
                    Tool::Selected(..) => {
                        ui.ctx().set_cursor_icon(CursorIcon::Grab);
                    }
                    Tool::MovingText(..) => {
                        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                    }
                    _ => {
                        ui.ctx().set_cursor_icon(CursorIcon::Default);
                    }
                }
            }
            if let Some(pos) = resp.interact_pointer_pos() {
                self.on_handle_interaction(&resp, &canvas, pos, &painter);
            }
            self.process_actions(ui);
            self.tool_specific_drawing(&canvas, &painter);
            if resp.double_clicked() {
                self.reset_zoom = true;
            }
        });
    }
    fn draw_preview_widget(&mut self, ui: &mut Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let desired_size = ui.available_size();
            let (resp, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());
            let canvas = resp.rect;
            let text_color = ui.style().visuals.strong_text_color();
            self.draw_rendered_schematic(&canvas, &painter, text_color);
            if resp.dragged_by(egui::PointerButton::Secondary) {
                self.drag_delta = Some(resp.drag_delta());
            }
            if resp.double_clicked() {
                self.reset_zoom = true;
            }
        });
    }
}

impl TabViewer for MyApp {
    type Tab = Tab;

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::Ascii => "ASCII".into(),
            Tab::Preview => "Preview".into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        let scene = Scene::new()
            .max_inner_size(vec2(1000.0, 800.0))
            .zoom_range(0.5..=3.0);
        let mut scene_rect = self.scene_rect;
        self.reset_zoom = false;
        match tab {
            Tab::Ascii => {
                self.ascii_control_panel(ui);
                scene.show(ui, &mut scene_rect, |ui| {
                    self.draw_ascii_widget(ui);
                });
            }
            Tab::Preview => {
                self.preview_control_panel(ui);
                scene.show(ui, &mut scene_rect, |ui| {
                    self.draw_preview_widget(ui);
                });
            }
        }
        self.scene_rect = scene_rect;
        if let Some(delta) = self.drag_delta.take() {
            self.scene_rect = self.scene_rect.translate(-delta);
        }
        if self.reset_zoom {
            self.scene_rect = Rect::ZERO;
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.resize_panel(ui);
                let mut dockstate = self.dock_state.clone();
                DockArea::new(&mut dockstate)
                    .style(Style::from_egui(ui.style().as_ref()))
                    .show_leaf_collapse_buttons(false)
                    .show_inside(ui, self);
                self.dock_state = dockstate;
                if let Some(txt) = std::mem::take(&mut self.copy_buffer) {
                    ctx.copy_text(txt);
                }
            })
        });
    }
}
