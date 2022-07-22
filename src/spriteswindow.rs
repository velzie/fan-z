use crate::app::toolbtn_ui;
use crate::mapwindow::Resize;
use crate::viewport::Viewport;
use crate::zvm::{self, ZEvent, ZVMState, ZVM};
use crate::FanzApp;
// use crate::app::
use array2d::Array2D;
use egui::{
    pos2, vec2, Align2, Color32, Id, Key, LayerId, Painter, Rect, RichText, Sense, Stroke, Vec2,
    Widget,
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct SpritesWindow {
    pub enabled: bool,

    #[serde(skip)]
    viewport: Viewport,

    selectedcolor: Color32,
    #[serde(skip)]
    selectedtool: Tool,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Tool {
    Pencil,
    Eraser,
    Picker,
    Rect,
    Line,
    Move,
}
impl Default for Tool {
    fn default() -> Self {
        Self::Pencil
    }
}
impl Default for SpritesWindow {
    fn default() -> Self {
        SpritesWindow {
            enabled: false,
            selectedcolor: Color32::TRANSPARENT,
            selectedtool: Tool::Pencil,
            viewport: Viewport::default(),
        }
    }
}
impl SpritesWindow {
    pub fn ui<'a>(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(50.0);
            ui.label("color: ");
            ui.color_edit_button_srgba(&mut self.selectedcolor);
            if ui.small_button("+").clicked() {
                app.cart.sprites.push(Sprite::new())
            }
            match app.cart.sprites.get_mut(app.selectedsprite) {
                Some(sprite) => {
                    let mut columns = sprite.data.num_columns();
                    let mut rows = sprite.data.num_rows();

                    ui.label("rows: ");
                    ui.add(egui::DragValue::new(&mut rows));
                    ui.label("columns: ");
                    ui.add(egui::DragValue::new(&mut columns));

                    while columns < sprite.data.row_len() {
                        sprite.data.popcolumn();
                    }
                    while columns > sprite.data.row_len() {
                        sprite.data.addcolumn(Color32::TRANSPARENT);
                    }

                    while rows < sprite.data.column_len() {
                        sprite.data.poprow();
                    }
                    while rows > sprite.data.column_len() {
                        sprite.data.addrow(Color32::TRANSPARENT);
                    }
                }
                None => (),
            };
        });
        match app.cart.sprites.get_mut(app.selectedsprite) {
            Some(sprite) => {
                let pixelsize = 32.0 / self.viewport.zoom;
                let height = ui.available_height();
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for variant in [
                            Tool::Pencil,
                            Tool::Eraser,
                            Tool::Picker,
                            Tool::Rect,
                            Tool::Line,
                            Tool::Move,
                        ] {
                            if sized_toolbtn_ui(
                                ui,
                                vec2(50.0, 50.0),
                                &format!("{:?}", variant),
                                self.selectedtool == variant,
                            )
                            .clicked()
                            {
                                self.selectedtool = variant;
                            }
                        }
                    });
                    let (resp, painter, start) = self.viewport.draw(
                        ui,
                        vec2(ui.available_width(), height),
                        vec2(
                            sprite.data.num_rows() as f32 * pixelsize,
                            sprite.data.num_columns() as f32 * pixelsize,
                        ),
                    );
                    for x in 0..sprite.data.num_rows() {
                        for y in 0..sprite.data.num_columns() {
                            let spritefill = sprite.data.get_mut(x, y).unwrap();
                            let squarerect = Rect::from_min_size(
                                start + vec2(x as f32 * pixelsize, y as f32 * pixelsize),
                                vec2(pixelsize, pixelsize),
                            );
                            for x2 in 0..4 {
                                for y2 in 0..4 {
                                    painter.rect_filled(
                                        Rect::from_min_size(
                                            start
                                                + vec2(
                                                    x as f32 * pixelsize
                                                        + x2 as f32 * pixelsize / 4.0,
                                                    y as f32 * pixelsize
                                                        + y2 as f32 * pixelsize / 4.0,
                                                ),
                                            vec2(pixelsize / 4.0, pixelsize / 4.0),
                                        ),
                                        0f32,
                                        if (x2 + y2) % 2 != 0 {
                                            Color32::LIGHT_GRAY
                                        } else {
                                            Color32::GRAY
                                        },
                                    );
                                }
                            }

                            painter.rect(
                                squarerect,
                                0f32,
                                spritefill.clone(),
                                Stroke::new(2.0, Color32::WHITE),
                            );

                            match &resp.hover_pos() {
                                Some(s) => {
                                    for e in &ui.input().events {
                                        match e {
                                            egui::Event::Scroll(s) => {
                                                self.viewport.zoom -= s.y / 100000.0;
                                            }
                                            _ => (),
                                        }
                                    }
                                    if squarerect.contains(*s) {
                                        match self.selectedtool {
                                            Tool::Pencil | Tool::Rect | Tool::Line => {
                                                painter.rect_filled(
                                                    squarerect.clone(),
                                                    0f32,
                                                    self.selectedcolor,
                                                );
                                            }
                                            Tool::Move => {}
                                            Tool::Eraser => {
                                                painter.rect_filled(
                                                    squarerect.clone(),
                                                    0f32,
                                                    Color32::BLACK,
                                                );
                                            }
                                            Tool::Picker => {
                                                resp.clone()
                                                    .on_hover_cursor(egui::CursorIcon::Crosshair);
                                                painter.rect_filled(
                                                    squarerect.clone(),
                                                    0f32,
                                                    Color32::DEBUG_COLOR,
                                                );
                                            }
                                        }

                                        match self.selectedtool {
                                            Tool::Pencil => {
                                                if ui.input().key_down(Key::Space) && resp.dragged()
                                                {
                                                    resp.clone()
                                                        .on_hover_cursor(egui::CursorIcon::Grab);
                                                    self.viewport.offset -=
                                                        resp.drag_delta() * self.viewport.zoom;
                                                } else if resp
                                                    .clicked_by(egui::PointerButton::Secondary)
                                                    || resp
                                                        .dragged_by(egui::PointerButton::Secondary)
                                                {
                                                    *spritefill = Color32::TRANSPARENT;
                                                } else if resp
                                                    .clicked_by(egui::PointerButton::Primary)
                                                    || resp.dragged_by(egui::PointerButton::Primary)
                                                {
                                                    *spritefill = self.selectedcolor;
                                                }
                                            }
                                            Tool::Eraser => {
                                                if resp.clicked_by(egui::PointerButton::Primary)
                                                    || resp.dragged_by(egui::PointerButton::Primary)
                                                {
                                                    *spritefill = Color32::TRANSPARENT;
                                                }
                                            }
                                            Tool::Picker => {
                                                if resp.drag_started() {
                                                    self.selectedcolor = *spritefill;
                                                    self.selectedtool = Tool::Pencil;
                                                }
                                            }
                                            Tool::Move => {
                                                resp.clone()
                                                    .on_hover_cursor(egui::CursorIcon::Grab);
                                                if resp.dragged() {
                                                    self.viewport.offset -=
                                                        resp.drag_delta() * self.viewport.zoom;
                                                }
                                            }
                                            Tool::Rect | Tool::Line => todo!(),
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                });
            }
            None => (),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Sprite {
    pub data: Array2D<Color32>,
}
impl Sprite {
    pub fn new() -> Sprite {
        Sprite {
            data: Array2D::filled_with(Color32::TRANSPARENT, 8, 8),
        }
    }
}

pub fn sized_toolbtn_ui(ui: &mut egui::Ui, size: Vec2, text: &str, on: bool) -> egui::Response {
    let (id, rect) = ui.allocate_space(size);
    let response = ui.interact(rect, id, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let mut color = visuals.bg_fill;
        if on {
            color = Color32::BLACK;
        }
        ui.painter().rect(
            rect,
            5.0,
            color,
            Stroke::new(if response.hovered() { 1.0 } else { 0.0 }, Color32::WHITE),
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            egui::FontId {
                size: 15.0,
                family: egui::FontFamily::Proportional,
            },
            Color32::WHITE,
        );
    }

    response
}
