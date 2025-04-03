#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

// Needs:
// SVG export - maybe with roughr

use std::collections::{HashMap, HashSet, VecDeque};

mod lib;

use crate::lib::{rect::Rectangle, tc::TextCoordinate};

/**
 *      .----
 *      |
 *   ---| i.data
 *      |
 *   ---| i.prev
 *      |
 *   -->| next
 *      |
 *      .____
 *
 * For each pin, we need
 *   - name                 } - these come from the struct?
 *   - direction (in/out)   }
 *   - interface (optional)   - encoded in the name?
 *   - side (l/r)          
 *   - offset from center (+/-)
 *
 * For the overall block we also need
 *   - padx (padding between the l/r labels)
 *   - pady (padding between the top and bottom labels)
 *
 * Simplest solution
 *   - Add a pin tool (or tools?  Maybe an input tool and an output tool)
 *   - Select a point on a rectangle boundary (l or right)
 *   - Enter the name of the pin
 *
 *  When pasting, extract pins from the symbol?
 */
use eframe::egui;
use egui::{
    Align2, Button, Color32, CursorIcon, DragValue, Event, FontId, Key, Modifiers, Painter, Pos2,
    Rect, Sense, UiBuilder, epaint::PathStroke, vec2,
};
use lib::{Resize, action::Action, analyze::get_rectangles, text_buffer::TextBuffer};

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

#[derive(Copy, Clone, Debug, PartialEq)]
enum Shape {
    Rectangle(Rect),
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
    selected_text: TextBuffer,
    tool: Tool,
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
    resize: Option<Resize>,
}

const INITIAL_TEXT: &str = "
     +---------------------+
     |                     |
    >| data           data |o
     |                     |
    o| full           next |>
     |                     |
    o| overflow  underflow |o   
     |                     |
     +---------------------+
";

impl Default for MyApp {
    fn default() -> Self {
        let num_rows = 40;
        let num_cols = 100;
        let mut text = TextBuffer::new(num_rows, num_cols);
        text.paste(INITIAL_TEXT, TextCoordinate { x: 20, y: 5 });
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
        self.snapshots.push_back(Snapshot {
            text: self.text.clone(),
            selected_text: self.selected_text.clone(),
            tool: self.tool.clone(),
        });
    }
    fn set_text(&mut self, ch: char, position: &TextCoordinate) {
        self.text.set_text(position, Some(ch));
    }
    fn clear_text(&mut self, position: &TextCoordinate) {
        self.text.set_text(position, None);
    }
    fn on_drag_start(&mut self, tc: TextCoordinate) {
        match &self.tool {
            Tool::Selection(None) => {
                self.tool = Tool::Selection(Some(tc));
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
                self.snapshot();
                self.selected_text = self.text.clone();
                self.text.clear_rectangle(selection);
                self.tool = Tool::Selected(selection);
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
                    swap_buf.set_text(&new_pos, selection);
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
                    self.text.set_text(&pos, selection);
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
        match action {
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
                self.set_text('-', &cursor);
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
                self.set_text('-', &cursor);
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
                self.set_text('|', &cursor);
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
                self.set_text('|', &cursor);
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
    }
    fn on_action(&mut self, action: Action) {
        match &self.tool {
            Tool::Text(Some(text_state)) => {
                self.on_action_with_text(*text_state, action);
            }
            Tool::Selection(None) => match action {
                Action::Char('t') => self.tool = Tool::Text(None),
                Action::Copy => {
                    eprintln!("Copy of buffer");
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
                    self.text.set_text(&pos, selection);
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
            self.selected_text = buf.selected_text;
            self.tool = buf.tool;
        }
    }
    fn redo(&mut self) {
        if let Some(buf) = self.futures.pop() {
            self.snapshots.push_back(buf.clone());
            self.text = buf.text;
            self.selected_text = buf.selected_text;
            self.tool = buf.tool;
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("⚙").clicked() {
                        self.resize = Some(Resize {
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
                    if ui.button("☸").clicked() {
                        let tic = std::time::Instant::now();
                        let rectangles = get_rectangles(&self.text);
                        eprintln!("Elapsed {:?}", tic.elapsed());
                        eprintln!("{:?}", rectangles);
                    }
                });
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
                egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    let desired_size = ui.available_size();
                    let (resp, painter) =
                        ui.allocate_painter(desired_size, Sense::click_and_drag());
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
                            self.on_hover(Some(text_coordinate));
                        } else {
                            self.on_hover(None);
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
                    if let Tool::Text(Some(TextState { origin: _, cursor })) = self.tool {
                        let center = self.map_text_coordinate_to_cell_center(&canvas, &cursor);
                        let rect = Rect::from_center_size(center, vec2(delta_x, delta_y));
                        painter.rect_stroke(
                            rect,
                            0.5,
                            (1.0, Color32::LIGHT_YELLOW),
                            egui::StrokeKind::Middle,
                        );
                    }
                    if let Tool::Selected(selection_box) = self.tool {
                        for (coord, ch) in self.selected_text.iter() {
                            if selection_box.contains(&coord) {
                                let center =
                                    self.map_text_coordinate_to_cell_center(&canvas, &coord);
                                let monospace = FontId::monospace(10.0);
                                painter.text(
                                    center,
                                    Align2::CENTER_CENTER,
                                    ch,
                                    monospace,
                                    Color32::GREEN.linear_multiply(0.5),
                                );
                            }
                        }
                    }
                    if let Tool::MovingText(MoveState {
                        selection,
                        origin,
                        move_pos,
                    }) = self.tool
                    {
                        let bbox_shifted = selection.shifted(origin, move_pos);
                        for (coord, ch) in self.selected_text.iter() {
                            let coord = coord.shifted(origin, move_pos);
                            if bbox_shifted.contains(&coord) {
                                let center =
                                    self.map_text_coordinate_to_cell_center(&canvas, &coord);
                                let monospace = FontId::monospace(10.0);
                                painter.text(
                                    center,
                                    Align2::CENTER_CENTER,
                                    ch,
                                    monospace,
                                    Color32::GREEN,
                                );
                            }
                        }
                    }
                });
                match &self.tool {
                    Tool::Text(_) => {
                        ctx.set_cursor_icon(CursorIcon::Text);
                    }
                    Tool::Selected(..) => {
                        ctx.set_cursor_icon(CursorIcon::Grab);
                    }
                    Tool::MovingText(..) => {
                        ctx.set_cursor_icon(CursorIcon::Grabbing);
                    }
                    _ => {
                        ctx.set_cursor_icon(CursorIcon::Default);
                    }
                }
                if let Some(txt) = std::mem::take(&mut self.copy_buffer) {
                    ctx.copy_text(txt);
                }
            })
        });
    }
}
